// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    FromExt, OPTIMAL_THREADS, disc_stream::SplitFileDiscStream, import_entry::ImportEntry,
    multi_disc::open_multi_disc_files,
};
use anyhow::{Context, Result, anyhow};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_read::{SharedSplitFileReader, SplitFileReader};
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use tempfile::tempfile;
use twbm_common::config_contents::ConfigContents;
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
pub const BUF_SIZE: usize = 4 * 1024 * 1024; // 4 MiB

fn get_disc_files(path: &Path) -> Result<Vec<File>> {
    let is_zip = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

    if is_zip {
        let mut f = File::open(path)?;
        let mut zip = ZipArchive::new(&mut f)?;
        let mut entry = zip.by_index(0)?;

        let mut tmp = tempfile()?;
        io::copy(&mut entry, &mut tmp)?;

        Ok(vec![tmp])
    } else {
        open_multi_disc_files(path)
    }
}

pub fn import_game(
    entry: &ImportEntry,
    config: &ConfigContents,
    is_fat32: bool,
    send_progress: impl Fn(String),
) -> Result<String> {
    let filename = entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    let mount_point = config.mount_point.as_ref().context("No mount point")?;

    send_progress(format!("›  Opening {filename}"));

    let disc_files = get_disc_files(entry.path())?;

    let must_split = entry.meta().is_wii() && (config.always_split || is_fat32);
    let split_size = must_split.then_some(SPLIT_SIZE);

    let parent_dir_name = if entry.meta().is_wii() {
        "wbfs"
    } else {
        "games"
    };

    let parent_dir = mount_point.join(parent_dir_name);
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
                &config.wii_output_format,
                &config.gc_output_format,
                must_split,
            )
        },
        split_size,
    )?;

    let out_format_str = if entry.meta().is_wii() {
        config.wii_output_format.as_str()
    } else {
        config.gc_output_format.as_str()
    };

    let out_format = Format::from_ext(out_format_str).context("Invalid output format")?;

    if out_format_str.eq_ignore_ascii_case(entry.meta().format().as_str()) {
        // if the input is already in the right format, just copy it

        let disc_file_reader = SplitFileReader::new(disc_files)?;
        let total_size = disc_file_reader.total_size();
        let mut reader = BufReader::with_capacity(BUF_SIZE, disc_file_reader);
        let mut written = 0;
        let mut prev_percentage = 100;

        loop {
            let buf = reader.fill_buf()?;
            let buf_len = buf.len();
            if buf_len == 0 {
                break;
            }

            out_writer.write_all(buf)?;

            reader.consume(buf_len);
            written += buf_len as u64;

            let progress_percentage = written * 100 / total_size;
            if progress_percentage != prev_percentage {
                send_progress(format!(
                    "⤓  Importing {}  {progress_percentage:02}%",
                    entry.meta().game_title()
                ));

                prev_percentage = progress_percentage;
            }
        }

        out_writer.finalize(None)?;
    } else {
        let disc_reader = {
            let disc_opts = DiscOptions {
                preloader_threads: OPTIMAL_THREADS.preloader,
                ..Default::default()
            };

            let disc_file_reader = SharedSplitFileReader::new(disc_files)?;
            let stream = SplitFileDiscStream(disc_file_reader);
            DiscReader::new_stream(Box::new(stream), &disc_opts)?
        };

        let mut out_writer = BufWriter::with_capacity(BUF_SIZE, out_writer);
        let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(out_format))?;

        let options = ProcessOptions {
            processor_threads: OPTIMAL_THREADS.processor,
            scrub: if config.scrub_update_partition {
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
                    send_progress(format!(
                        "⤓  Importing {}  {progress_percentage:02}%",
                        entry.meta().game_title()
                    ));

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &options,
        )?;

        let out_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to flush BufWriter"))?;

        if !finalization.header.is_empty() {
            out_writer.finalize(Some(&finalization.header))?;
        }
    }

    if config.remove_sources_games {
        fs::remove_file(entry.path())?;
    }

    Ok(format!("Imported {}", entry.meta().game_title()))
}

pub fn get_filename(
    game_id: GameID,
    is_wii: bool,
    part: usize,
    disc_num: u8,
    wii_output_format: &str,
    gc_output_format: &str,
    must_split: bool,
) -> String {
    if is_wii {
        match wii_output_format {
            "iso" => {
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
            "ciso" => match disc_num {
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

    fs::create_dir_all(&path)?;
    Ok(path)
}
