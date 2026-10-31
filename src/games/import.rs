// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::{
        ImportEntry,
        disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files},
    },
    tasks::{self, Reporter, TaskControl, WorkerEvent},
    util::{drive_state::DriveState, misc::OPTIMAL_THREADS},
};
use anyhow::{Context, Result, anyhow};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use smol::stream::Stream;
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    io::{Read, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use which_fs::FsKind;
use wii_disc_info::game_id::GameID;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB

/// Writes the game into its folder on the drive. On failure or cancel, whatever this import
/// wrote is removed again, so no half-written game stays behind.
fn perform_blocking(
    entry: &ImportEntry,
    config: &Config,
    drive: &DriveState,
    control: &TaskControl,
    reporter: &mut Reporter,
) -> Result<()> {
    let filename = entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    reporter.progress(0, None, || format!("Opening {filename}"));

    let parent_dir_name = if entry.meta().is_wii() {
        "wbfs"
    } else {
        "games"
    };

    let game_dir = config
        .mount_point()
        .join(parent_dir_name)
        .join(make_game_dir_name(
            entry.meta().game_id(),
            entry.meta().game_title(),
        ));

    // the second disc of a game goes into the folder of the first one
    let dir_existed = game_dir.exists();
    let files_before = list_files(&game_dir);
    std::fs::create_dir_all(&game_dir)?;

    if let Err(e) = write_game(entry, config, drive, &game_dir, control, reporter) {
        if dir_existed {
            for file in list_files(&game_dir) {
                if !files_before.contains(&file) {
                    let _ = std::fs::remove_file(file);
                }
            }
        } else {
            let _ = std::fs::remove_dir_all(&game_dir);
        }

        return Err(e);
    }

    if config.remove_sources_games() {
        std::fs::remove_file(entry.path())?;
    }

    Ok(())
}

fn list_files(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| entries.filter_map(|e| Some(e.ok()?.path())).collect())
        .unwrap_or_default()
}

fn write_game(
    entry: &ImportEntry,
    config: &Config,
    drive: &DriveState,
    game_dir: &Path,
    control: &TaskControl,
    reporter: &mut Reporter,
) -> Result<()> {
    let mut files = retrieve_multi_disc_files(entry.path())?;

    let is_fat32 = if let DriveState::Loaded(info) = drive {
        info.fs_kind() == FsKind::Fat32
    } else {
        true
    };

    let must_split = entry.meta().is_wii() && (is_fat32 || config.always_split());
    let split_size = must_split.then_some(SPLIT_SIZE);

    let out_writer = SplitWriter::create(
        game_dir,
        |part| {
            get_filename(
                entry.meta().game_id(),
                entry.meta().is_wii(),
                part,
                entry.meta.disc_number(),
                config.wii_output_format(),
                config.gc_output_format(),
                must_split,
            )
        },
        split_size,
    )?;

    let out_format = if entry.meta().is_wii() {
        config.wii_output_format()
    } else {
        config.gc_output_format()
    };

    let title = entry.display_title();
    let message = || format!("Importing {title}");

    if out_format == wii_disc_info_to_nod_format(entry.meta().format()) {
        // if the input is already in the right format, just copy it

        // move here so it gets dropped
        let mut out_writer = out_writer;

        let mut to_write = 0;
        for file in &files {
            to_write += file.metadata()?.len();
        }

        let mut written = 0;
        let mut buf = vec![0; 0x8000].into_boxed_slice();

        for file in &mut files {
            loop {
                control.checkpoint()?;

                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }

                out_writer.write_all(&buf[..n])?;
                written += n as u64;

                reporter.progress(written, Some(to_write), message);
            }
        }

        out_writer.flush()?;
    } else {
        let stream = SharedMultiFileReader::new(files)?;

        let disc_reader = DiscReader::new_stream(
            Box::new(stream),
            &DiscOptions {
                preloader_threads: OPTIMAL_THREADS.preloader,
                ..Default::default()
            },
        )?;

        let mut out_writer = std::io::BufWriter::with_capacity(0x8000, out_writer);
        let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(out_format))?;

        let options = ProcessOptions {
            processor_threads: OPTIMAL_THREADS.processor,
            scrub: if config.scrub_update_partition() {
                ScrubLevel::UpdatePartition
            } else {
                ScrubLevel::None
            },
            digest_crc32: true,
            digest_md5: false,
            digest_sha1: true,
            digest_xxh64: true,
        };

        let finalization = disc_writer.process(
            |data, progress, total| {
                control.checkpoint().map_err(std::io::Error::other)?;
                out_writer.write_all(&data)?;
                reporter.progress(progress, Some(total), message);
                Ok(())
            },
            &options,
        );

        // nod wraps the error; the task manager knows it was cancelled anyway
        control.checkpoint()?;
        let finalization = finalization?;

        let mut out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner writer"))?;

        if !finalization.header.is_empty() {
            out_writer.write_header(&finalization.header)?;
        }

        out_writer.flush()?;
    }

    Ok(())
}

/// Imports a game to the drive as a task: progress in bytes, pause and cancel through `control`.
pub fn import_game(
    entry: ImportEntry,
    config: Config,
    drive: DriveState,
    control: TaskControl,
) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(move |reporter| {
        perform_blocking(&entry, &config, &drive, &control, reporter)
            .map(|()| format!("Imported {}", entry.display_title()))
            .map_err(|e| e.to_string())
    })
}

pub fn get_filename(
    game_id: GameID,
    is_wii: bool,
    part: usize,
    disc_num: u8,
    wii_output_format: Format,
    gc_output_format: Format,
    must_split: bool,
) -> String {
    if is_wii {
        match wii_output_format {
            Format::Iso => {
                if must_split {
                    format!("{game_id}.part{part}.iso")
                } else {
                    format!("{game_id}.iso")
                }
            }
            _ => match part {
                0 => format!("{game_id}.wbfs"),
                n => format!("{game_id}.wbf{n}"),
            },
        }
    } else {
        match gc_output_format {
            Format::Ciso => match disc_num {
                0 => "game.ciso".to_string(),
                n => format!("disc{}.ciso", n + 1),
            },
            _ => match disc_num {
                0 => "game.iso".to_string(),
                n => format!("disc{}.iso", n + 1),
            },
        }
    }
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !#$%&'()+,-.;=@^_`{}~".contains(c)
}

pub fn sanitize_title(ascii_title: &str) -> String {
    let mut title = ascii_title
        .chars()
        .skip_while(|c| !c.is_ascii_alphanumeric())
        .filter(|&c| is_valid_char(c))
        .take(64)
        .collect::<String>();

    // Remove trailing whitespace in-place
    title.truncate(title.trim_end().len());

    if title.is_empty() {
        title.push_str("game");
    }

    title
}

pub fn make_game_dir_name(game_id: GameID, fallback_title: &str) -> String {
    let game_title = {
        let ascii_title = twbm_idmap::get_ascii_title(game_id).unwrap_or(fallback_title);
        sanitize_title(ascii_title)
    };

    format!("{game_title} [{game_id}]")
}

fn wii_disc_info_to_nod_format(format: wii_disc_info::Format) -> Format {
    match format {
        wii_disc_info::Format::Wbfs => Format::Wbfs,
        wii_disc_info::Format::Ciso => Format::Ciso,
        wii_disc_info::Format::Iso => Format::Iso,
        wii_disc_info::Format::Rvz => Format::Rvz,
        wii_disc_info::Format::Wia => Format::Wia,
        wii_disc_info::Format::Gcz => Format::Gcz,
        wii_disc_info::Format::Tgc => Format::Tgc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::stream::StreamExt;

    /// A GameCube disc image with only a header: enough to be recognised and copied.
    fn fake_gc_disc(dir: &Path) -> PathBuf {
        let mut disc = vec![0u8; 3 * 0x8000];
        disc[..6].copy_from_slice(b"GTST01");
        disc[0x1c..0x20].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);
        disc[0x20..0x60].copy_from_slice(&[b' '; 0x40]);
        disc[0x20..0x29].copy_from_slice(b"Test Game");
        for (i, byte) in disc.iter_mut().enumerate().skip(0x60) {
            *byte = i as u8;
        }

        let path = dir.join("Test Game.iso");
        std::fs::write(&path, disc).unwrap();
        path
    }

    fn import(source: &Path, drive: &Path, control: TaskControl) -> Result<String, String> {
        let entry = smol::block_on(ImportEntry::new(source.to_path_buf())).unwrap();
        let mut config = Config::default();
        let _ = config.set_mount_point(drive.to_path_buf());

        let events: Vec<WorkerEvent> =
            smol::block_on(import_game(entry, config, DriveState::NotLoaded, control).collect());

        match events.last() {
            Some(WorkerEvent::Finished(result)) => result.clone(),
            other => panic!("no result: {other:?}"),
        }
    }

    #[test]
    fn copies_the_game_into_its_folder() {
        let source = tempfile::tempdir().unwrap();
        let drive = tempfile::tempdir().unwrap();
        let disc = fake_gc_disc(source.path());

        let result = import(&disc, drive.path(), TaskControl::new());
        assert!(result.is_ok(), "{result:?}");

        let game_dir = drive.path().join("games").join("Test Game [GTST01]");
        assert_eq!(
            std::fs::read(game_dir.join("game.iso")).unwrap(),
            std::fs::read(&disc).unwrap()
        );
    }

    #[test]
    fn a_cancelled_import_leaves_nothing_behind() {
        let source = tempfile::tempdir().unwrap();
        let drive = tempfile::tempdir().unwrap();
        let disc = fake_gc_disc(source.path());

        let game_dir = drive.path().join("games").join("Test Game [GTST01]");

        let control = TaskControl::new();
        control.cancel();
        assert!(import(&disc, drive.path(), control).is_err());
        assert!(!game_dir.exists());

        // a folder that was there before (the first disc of the game) stays, with its files
        std::fs::create_dir_all(&game_dir).unwrap();
        std::fs::write(game_dir.join("disc1.iso"), b"first disc").unwrap();

        let control = TaskControl::new();
        control.cancel();
        assert!(import(&disc, drive.path(), control).is_err());
        let left: Vec<_> = list_files(&game_dir);
        assert_eq!(left, vec![game_dir.join("disc1.iso")]);
    }
}
