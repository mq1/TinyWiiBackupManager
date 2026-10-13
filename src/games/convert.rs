// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::export::ext_to_format, long_operation::LongOperationState, util::misc::OPTIMAL_THREADS,
};
use anyhow::{Context, Result, ensure};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use smol::stream::Stream;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Seek, Write},
    path::{Path, PathBuf},
};

// Conversion may replace a file confirmed by the save dialog, but only once the
// entire new image and its final header have been written successfully.
fn write_staged(
    out_path: &Path,
    write: impl FnOnce(&mut BufWriter<&mut File>) -> Result<()>,
) -> Result<()> {
    let parent = out_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".twbm-convert-")
        .tempfile_in(parent)
        .context("creating temporary conversion file")?;
    let written = (|| -> Result<()> {
        let mut writer = BufWriter::new(temporary.as_file_mut());
        write(&mut writer)?;
        writer.flush()?;
        drop(writer);
        temporary.as_file().sync_all()?;
        Ok(())
    })();
    if let Err(error) = written {
        return Err(discard_temporary(temporary, error));
    }
    if let Err(error) = temporary.persist(out_path) {
        return Err(discard_temporary(error.file, error.error.into()))
            .with_context(|| format!("saving {}", out_path.display()));
    }
    Ok(())
}

fn discard_temporary(temporary: tempfile::NamedTempFile, error: anyhow::Error) -> anyhow::Error {
    let path = temporary.path().to_path_buf();
    match temporary.close() {
        Ok(()) => error,
        Err(cleanup) => error.context(format!(
            "Could not remove incomplete conversion {}: {cleanup}",
            path.display()
        )),
    }
}

fn perform_blocking(
    disc_path: PathBuf,
    out_path: PathBuf,
    tx: &smol::channel::Sender<LongOperationState>,
) -> Result<String> {
    if out_path.exists() {
        let destination = out_path.canonicalize()?;
        for source in DiscReader::split_file_paths(&disc_path)? {
            ensure!(
                source.canonicalize()? != destination,
                "The conversion destination cannot be the source image or one of its parts"
            );
        }
    }

    let filename = out_path
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    let ext = out_path
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ext_to_format)
        .context("invalid extension")?;

    let format_opts = FormatOptions::new(ext);

    let disc_reader = DiscReader::new(
        &disc_path,
        &DiscOptions {
            preloader_threads: OPTIMAL_THREADS.preloader,
            ..Default::default()
        },
    )?;

    let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;
    write_staged(&out_path, |out_writer| {
        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage =
                    progress.saturating_mul(100).checked_div(total).unwrap_or(0);
                if progress_percentage != prev_percentage {
                    let _ = tx.try_send(LongOperationState::Progress(format!(
                        "Converting into {filename}  {progress_percentage:02}%",
                    )));

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &ProcessOptions {
                processor_threads: OPTIMAL_THREADS.processor,
                scrub: ScrubLevel::None,
                digest_crc32: true,
                digest_md5: false,
                digest_sha1: true,
                digest_xxh64: true,
            },
        )?;

        if !finalization.header.is_empty() {
            out_writer.rewind()?;
            out_writer.write_all(&finalization.header)?;
        }
        Ok(())
    })?;
    Ok(filename.to_string())
}

pub fn convert_game(
    disc_path: PathBuf,
    out_path: PathBuf,
) -> impl Stream<Item = LongOperationState> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = match perform_blocking(disc_path, out_path, &tx) {
            Ok(filename) => LongOperationState::Finished(format!("Converted {filename}")),
            Err(e) => LongOperationState::Errored(e.to_string()),
        };

        let _ = tx.send_blocking(exit);
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_disc() -> Vec<u8> {
        let mut bytes = vec![0u8; 0x10000];
        bytes[..6].copy_from_slice(b"GZZE01");
        bytes[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        bytes
    }

    #[test]
    fn conversion_rejects_source_as_destination_without_changing_it() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("source.iso");
        let source = synthetic_disc();
        std::fs::write(&path, &source).unwrap();
        let (tx, _rx) = smol::channel::bounded(1);

        let result = perform_blocking(path.clone(), path.clone(), &tx);

        assert_eq!(
            std::fs::read(&path).unwrap(),
            source,
            "conversion modified its source"
        );
        assert!(
            result.is_err(),
            "conversion must reject using its source as the output"
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[test]
    fn conversion_rejects_split_source_part_as_destination_without_changing_it() {
        let directory = tempfile::tempdir().unwrap();
        let primary = directory.path().join("source.part0.iso");
        let secondary = directory.path().join("source.part1.iso");
        let tertiary = directory.path().join("source.part2.iso");
        let output = directory.path().join("converted.iso");
        let mut source = synthetic_disc();
        source[0x8000..].fill(0x5a);
        std::fs::write(&primary, &source[..0x8000]).unwrap();
        std::fs::write(&secondary, &source[0x8000..0xc000]).unwrap();
        std::fs::write(&tertiary, &source[0xc000..]).unwrap();
        let (tx, _rx) = smol::channel::bounded(1);

        // First prove that the fixture is readable and every part is consumed.
        perform_blocking(primary.clone(), output.clone(), &tx).unwrap();
        assert_eq!(std::fs::read(output).unwrap(), source);

        let result = perform_blocking(primary.clone(), secondary.clone(), &tx);

        assert!(
            std::fs::read(primary).unwrap() == source[..0x8000],
            "conversion modified the primary source part"
        );
        let secondary_after = std::fs::read(secondary).unwrap();
        assert!(
            secondary_after == source[0x8000..0xc000],
            "conversion replaced the secondary source part (expected 16384 bytes, got {})",
            secondary_after.len()
        );
        assert!(
            std::fs::read(tertiary).unwrap() == source[0xc000..],
            "conversion modified the tertiary source part"
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "The conversion destination cannot be the source image or one of its parts"
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 4);
    }

    #[test]
    fn failed_conversion_preserves_existing_output_and_removes_partial_output() {
        for existing in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("destination.iso");
            if existing {
                std::fs::write(&path, b"previous image").unwrap();
            }

            let error = write_staged(&path, |writer| {
                writer.write_all(b"incomplete new image")?;
                anyhow::bail!("injected conversion error")
            })
            .unwrap_err();

            assert!(error.to_string().contains("injected conversion error"));
            if existing {
                assert_eq!(std::fs::read(&path).unwrap(), b"previous image");
            } else {
                assert!(!path.exists());
            }
            assert_eq!(
                std::fs::read_dir(directory.path()).unwrap().count(),
                usize::from(existing)
            );
        }
    }

    #[test]
    fn completed_conversion_replaces_confirmed_output_without_changing_source() {
        let directory = tempfile::tempdir().unwrap();
        let source_path = directory.path().join("source.iso");
        let output_path = directory.path().join("destination.iso");
        let source = synthetic_disc();
        std::fs::write(&source_path, &source).unwrap();
        std::fs::write(&output_path, b"previous image").unwrap();
        let (tx, _rx) = smol::channel::bounded(1);

        perform_blocking(source_path.clone(), output_path.clone(), &tx).unwrap();

        assert_eq!(std::fs::read(source_path).unwrap(), source);
        assert_eq!(std::fs::read(output_path).unwrap(), source);
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }
}
