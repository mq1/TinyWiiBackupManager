// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::export::ext_to_format,
    tasks::{self, Reporter, TaskControl, WorkerEvent},
    util::misc::OPTIMAL_THREADS,
};
use anyhow::{Context, Result};
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

/// Writes the disc at `disc_path` to `out_path`, in the format its extension names. On failure
/// or cancel the half-written file is removed again.
pub fn write_disc_blocking(
    disc_path: &Path,
    out_path: &Path,
    control: &TaskControl,
    reporter: &mut Reporter,
    message: impl Fn() -> String,
) -> Result<()> {
    let format = out_path
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ext_to_format)
        .context("invalid extension")?;

    reporter.progress(0, None, &message);

    let disc_reader = DiscReader::new(
        disc_path,
        &DiscOptions {
            preloader_threads: OPTIMAL_THREADS.preloader,
            ..Default::default()
        },
    )?;

    let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(format))?;
    let mut out_writer = BufWriter::new(File::create(out_path)?);

    let result = (|| -> Result<()> {
        let finalization = disc_writer.process(
            |data, progress, total| {
                control.checkpoint().map_err(std::io::Error::other)?;
                out_writer.write_all(&data)?;
                reporter.progress(progress, Some(total), &message);
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
        );

        // nod wraps the error; the task manager knows it was cancelled anyway
        control.checkpoint()?;
        let finalization = finalization?;

        if !finalization.header.is_empty() {
            out_writer.rewind()?;
            out_writer.write_all(&finalization.header)?;
        }

        out_writer.flush()?;
        Ok(())
    })();

    if result.is_err() {
        drop(out_writer);
        let _ = std::fs::remove_file(out_path);
    }

    result
}

/// Converts a disc image into another format as a task.
pub fn convert_game(
    disc_path: PathBuf,
    out_path: PathBuf,
    control: TaskControl,
) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(move |reporter| {
        let filename = out_path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or_default()
            .to_string();

        write_disc_blocking(&disc_path, &out_path, &control, reporter, || {
            format!("Converting into {filename}")
        })
        .map(|()| format!("Converted {filename}"))
        .map_err(|e| e.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::stream::StreamExt;

    /// A GameCube disc image with only a header: enough for nod to read it.
    fn fake_gc_disc(dir: &Path) -> PathBuf {
        let mut disc = vec![0u8; 3 * 0x8000];
        disc[..6].copy_from_slice(b"GTST01");
        disc[0x1c..0x20].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);
        disc[0x20..0x29].copy_from_slice(b"Test Game");

        let path = dir.join("Test Game.iso");
        std::fs::write(&path, disc).unwrap();
        path
    }

    fn convert(disc: &Path, out: &Path, control: TaskControl) -> Result<String, String> {
        let events: Vec<WorkerEvent> =
            smol::block_on(convert_game(disc.to_path_buf(), out.to_path_buf(), control).collect());

        match events.last() {
            Some(WorkerEvent::Finished(result)) => result.clone(),
            other => panic!("no result: {other:?}"),
        }
    }

    #[test]
    fn converts_into_the_format_of_the_extension() {
        let dir = tempfile::tempdir().unwrap();
        let disc = fake_gc_disc(dir.path());
        let out = dir.path().join("Test Game.ciso");

        let result = convert(&disc, &out, TaskControl::new());
        assert_eq!(result, Ok("Converted Test Game.ciso".to_string()));
        assert!(std::fs::metadata(&out).unwrap().len() > 0);
    }

    #[test]
    fn a_cancelled_conversion_leaves_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let disc = fake_gc_disc(dir.path());
        let out = dir.path().join("Test Game.ciso");

        let control = TaskControl::new();
        control.cancel();
        assert!(convert(&disc, &out, control).is_err());
        assert!(!out.exists());
        assert!(disc.exists());
    }
}
