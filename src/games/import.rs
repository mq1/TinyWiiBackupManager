// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::{
        ImportEntry,
        conversion_state::ConversionState,
        disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files},
    },
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

const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB

fn perform_blocking(
    entry: &ImportEntry,
    config: &Config,
    drive: &DriveState,
    tx: &smol::channel::Sender<ConversionState>,
) -> Result<()> {
    let filename = entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    tx.send_blocking(ConversionState::Progress(format!("›  Opening {filename}")))?;

    let mut files = retrieve_multi_disc_files(entry.path())?;

    let is_fat32 = if let DriveState::Loaded(info) = drive {
        info.fs_kind() == FsKind::Fat32
    } else {
        true
    };

    let must_split = entry.meta().is_wii() && (is_fat32 || config.always_split());
    let split_size = if must_split { Some(SPLIT_SIZE) } else { None };

    let parent_dir_name = if entry.meta().is_wii() {
        "wbfs"
    } else {
        "games"
    };

    let parent_dir = config.mount_point().join(parent_dir_name);
    let game_dir = make_game_dir(
        &parent_dir,
        entry.meta().game_id(),
        entry.meta().game_title(),
    )?;

    let mut out_writer = SplitWriter::create(
        &game_dir,
        |part| {
            get_filename(
                entry.meta().game_id(),
                entry.meta().is_wii(),
                part,
                entry.meta.disc_number(),
                config,
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

    if out_format == wii_disc_info_to_nod_format(entry.meta().format()) {
        // if the input is already in the right format, just copy it

        let mut to_write = 0;
        for file in &files {
            to_write += file.metadata()?.len();
        }

        let mut written = 0;
        let mut prev_percentage = 100;
        let mut buf = unsafe { Box::new_uninit_slice(0x8000).assume_init() };

        for file in &mut files {
            loop {
                let n = file.read(&mut buf)?;
                if n == 0 {
                    break;
                }

                out_writer.write_all(&buf[..n])?;
                written += n as u64;

                let progress_percentage = written * 100 / to_write;
                if progress_percentage != prev_percentage {
                    let _ = tx.try_send(ConversionState::Progress(format!(
                        "⤓  Importing {}  {progress_percentage:02}%",
                        entry.meta().game_title()
                    )));

                    prev_percentage = progress_percentage;
                }
            }
        }
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

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let _ = tx.try_send(ConversionState::Progress(format!(
                        "⤓  Importing {}  {progress_percentage:02}%",
                        entry.meta().game_title()
                    )));

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &options,
        )?;

        let mut out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner writer"))?;

        if !finalization.header.is_empty() {
            out_writer.write_header(&finalization.header)?;
        }

        out_writer.flush()?;
    }

    if config.remove_sources_games() {
        std::fs::remove_file(entry.path())?;
    }

    Ok(())
}

pub fn import_game(
    entry: ImportEntry,
    config: Config,
    drive: DriveState,
) -> impl Stream<Item = ConversionState> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = match perform_blocking(&entry, &config, &drive, &tx) {
            Ok(()) => ConversionState::Finished(format!("Imported {}", entry.meta().game_title())),
            Err(e) => ConversionState::Errored(e.to_string()),
        };

        tx.send_blocking(exit).expect("Channel should be open");
    });

    rx
}

fn get_filename(
    game_id: GameID,
    is_wii: bool,
    part: usize,
    disc_num: u8,
    config: &Config,
    must_split: bool,
) -> String {
    if is_wii {
        match config.wii_output_format() {
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
        match config.gc_output_format() {
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

fn make_game_dir(base_dir: &Path, game_id: GameID, fallback_title: &str) -> Result<PathBuf> {
    let dir_name = make_game_dir_name(game_id, fallback_title);
    let path = base_dir.join(dir_name);

    std::fs::create_dir_all(&path)?;
    Ok(path)
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
