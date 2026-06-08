// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, GcOutputFormat, WiiOutputFormat},
    drive_info::DriveInfo,
    game_id::GameID,
    util::{BUF_SIZE, HEADER_SIZE, SPLIT_SIZE, get_threads_num, make_game_dir_name},
};
use anyhow::{Result, anyhow, bail};
use crc32fast::Hasher;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_write::SplitWriter;
use std::{
    fs::{self, File},
    io::{BufWriter, Read, Write},
    path::PathBuf,
};
use which_fs::FsKind;
use zip::ZipArchive;

pub fn perform(
    mut in_path: PathBuf,
    config: &Config,
    drive_info: &DriveInfo,
    update_progress: &impl Fn(u8),
) -> Result<()> {
    let mut files_to_remove = Vec::new();

    let extracted = unzip(&mut in_path, update_progress, &mut files_to_remove)?;

    let (processor_threads, preloader_threads) = get_threads_num();

    let disc_opts = DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    };

    let disc_reader = DiscReader::new(&in_path, &disc_opts)?;
    let disc_header = disc_reader.header();
    let game_id = GameID::from(disc_header.game_id);
    let is_wii = disc_header.is_wii();
    let disc_num = disc_header.disc_num;

    let should_split =
        is_wii && (config.contents.always_split || (drive_info.fs_kind == FsKind::Fat32));

    let parent_dir_name = if is_wii { "wbfs" } else { "games" };
    let game_dir_name = make_game_dir_name(game_id, disc_header.game_title_str());
    let game_dir = config
        .contents
        .mount_point
        .join(parent_dir_name)
        .join(game_dir_name);

    let get_file_name = |i| {
        if is_wii {
            match config.contents.wii_output_format {
                WiiOutputFormat::Iso => {
                    if should_split {
                        format!("{}.part{i}.iso", game_id)
                    } else {
                        format!("{}.iso", game_id)
                    }
                }
                WiiOutputFormat::Wbfs => match i {
                    0 => format!("{}.wbfs", game_id),
                    n => format!("{}.wbf{n}", game_id),
                },
            }
        } else {
            match config.contents.gc_output_format {
                GcOutputFormat::Iso => match disc_num {
                    0 => "game.iso".to_string(),
                    n => format!("disc{}.iso", n + 1),
                },

                GcOutputFormat::Ciso => match disc_num {
                    0 => "game.ciso".to_string(),
                    n => format!("disc{}.ciso", n + 1),
                },
            }
        }
    };

    let out_format = match (
        is_wii,
        config.contents.wii_output_format,
        config.contents.gc_output_format,
    ) {
        (true, WiiOutputFormat::Iso, _) | (false, _, GcOutputFormat::Iso) => Format::Iso,
        (true, WiiOutputFormat::Wbfs, _) => Format::Wbfs,
        (false, _, GcOutputFormat::Ciso) => Format::Ciso,
    };

    let scrub = if config.contents.scrub_update_partition {
        ScrubLevel::UpdatePartition
    } else {
        ScrubLevel::None
    };

    let out_opts = FormatOptions::new(out_format);
    let process_opts = ProcessOptions {
        processor_threads,
        scrub,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: false,
        digest_xxh64: false,
    };

    let split_size = if should_split { Some(SPLIT_SIZE) } else { None };

    let hash_path = game_dir.join(format!("{game_id}.crc32"));

    fs::create_dir_all(&game_dir)?;
    let mut out_writer = BufWriter::with_capacity(
        BUF_SIZE,
        SplitWriter::create(&game_dir, get_file_name, split_size)?,
    );

    let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;
    let mut head_buffer = Vec::with_capacity(HEADER_SIZE);
    let mut hasher = Hasher::new();
    let mut last_percentage = 0;

    let finalization = disc_writer.process(
        |data, progress, size| {
            out_writer.write_all(&data)?;

            let remaining_in_head = HEADER_SIZE.saturating_sub(head_buffer.len());
            if remaining_in_head > 0 {
                let to_write = remaining_in_head.min(data.len());
                head_buffer.extend_from_slice(&data[..to_write]);
                hasher.update(&data[to_write..]);
            } else {
                hasher.update(&data);
            }

            let current_percentage = if extracted {
                (50 + progress * 50 / size) as u8
            } else {
                (progress * 100 / size) as u8
            };

            if current_percentage != last_percentage {
                update_progress(current_percentage);
                last_percentage = current_percentage;
            }

            Ok(())
        },
        &process_opts,
    )?;

    let mut split_writer = out_writer
        .into_inner()
        .map_err(|_| anyhow!("Failed to get inner split writer"))?;

    if !finalization.header.is_empty() {
        split_writer.write_header(&finalization.header)?;
        head_buffer[..finalization.header.len()].copy_from_slice(&finalization.header);
    }

    split_writer.flush()?;
    drop(split_writer);
    drop(disc_writer);

    let mut final_hasher = Hasher::new();
    final_hasher.update(&head_buffer);
    final_hasher.combine(&hasher);
    let checksum = final_hasher.finalize();
    fs::write(hash_path, format!("{checksum:08x}"))?;

    for path in files_to_remove {
        let _ = fs::remove_file(path);
    }

    Ok(())
}

fn unzip(
    in_path: &mut PathBuf,
    update_progress: &impl Fn(u8),
    files_to_remove: &mut Vec<PathBuf>,
) -> Result<bool> {
    let mut extracted = false;

    let is_zip = in_path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

    if !is_zip {
        return Ok(extracted);
    }

    let mut f = File::open(&in_path)?;
    let mut archive = ZipArchive::new(&mut f)?;
    let mut archived_disc = archive.by_index(0)?;

    let Some(parent) = in_path.parent() else {
        bail!("No parent dir found");
    };

    let new_in_path = parent.join(archived_disc.name());
    if !new_in_path.exists() {
        let size = archived_disc.size();
        let mut buf = vec![0u8; BUF_SIZE];
        let mut out = File::create(&new_in_path)?;
        let mut progress = 0;
        let mut last_percentage = 0;

        loop {
            let n = archived_disc.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;

            progress += n as u64;

            let current_percentage = (progress * 100 / size) as u8;

            if current_percentage != last_percentage {
                update_progress(current_percentage);
                last_percentage = current_percentage;
            }
        }

        out.flush()?;
        files_to_remove.push(new_in_path.clone());
        extracted = true;
    }

    *in_path = new_in_path;

    Ok(extracted)
}
