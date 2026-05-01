// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    drive_info::DriveInfo,
    util::{BUF_SIZE, HEADER_SIZE, SPLIT_SIZE, get_threads_num},
};
use anyhow::{Result, anyhow};
use crc32fast::Hasher;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_write::SplitWriter;
use std::{
    fs,
    io::{BufWriter, Write},
    path::Path,
};
use which_fs::FsKind;

pub fn perform(
    in_path: &Path,
    game_id: &str,
    config: &Config,
    drive_info: &DriveInfo,
    update_progress: &impl Fn(u8),
) -> Result<()> {
    let game_dir_path = in_path.parent().ok_or(anyhow!("No parent"))?;
    let game_dir_name = game_dir_path.file_name().ok_or(anyhow!("No file name"))?;
    let tmp_game_dir_name = format!("{} SCRUB", game_dir_name.to_string_lossy());
    let tmp_game_dir = game_dir_path.with_file_name(tmp_game_dir_name);
    let hash_path = tmp_game_dir.join(format!("{game_id}.crc32"));

    let (processor_threads, preloader_threads) = get_threads_num();
    let disc_opts = DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    };

    let process_opts = ProcessOptions {
        processor_threads,
        scrub: ScrubLevel::UpdatePartition,
        digest_crc32: false,
        digest_md5: false,
        digest_sha1: false,
        digest_xxh64: false,
    };

    let get_file_name = |i| match i {
        0 => format!("{game_id}.wbfs"),
        n => format!("{game_id}.wbf{n}"),
    };

    let should_split = config.contents.always_split || (drive_info.fs_kind == FsKind::Fat32);
    let split_size = if should_split { Some(SPLIT_SIZE) } else { None };

    let disc_reader = DiscReader::new(in_path, &disc_opts)?;
    let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(Format::Wbfs))?;

    fs::create_dir_all(&tmp_game_dir)?;
    let mut out_writer = BufWriter::with_capacity(
        BUF_SIZE,
        SplitWriter::create(&tmp_game_dir, get_file_name, split_size)?,
    );
    let mut hasher = Hasher::new();
    let mut head_buffer = Vec::with_capacity(HEADER_SIZE);

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

            let current_percentage = (progress * 100 / size) as u8;

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

    fs::remove_dir_all(game_dir_path)?;
    fs::rename(tmp_game_dir, game_dir_path)?;

    Ok(())
}
