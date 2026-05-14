// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    drive_info::DriveInfo,
    game::Game,
    util::{BUF_SIZE, HEADER_SIZE, SPLIT_SIZE, get_threads_num},
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
    ffi::OsStr,
    fs,
    io::{BufWriter, Write},
};
use which_fs::FsKind;

pub fn perform(
    game: &Game,
    config: &Config,
    drive_info: &DriveInfo,
    update_progress: &impl Fn(u8),
) -> Result<()> {
    let Some(in_path) = game.get_disc_path() else {
        bail!("Could not find disc file for {}", &game.title);
    };

    let Some(game_dir_name) = game.path.file_name().and_then(OsStr::to_str) else {
        bail!("Invalid filename");
    };

    let tmp_game_dir_name = format!("{game_dir_name} SCRUB");
    let tmp_game_dir = game.path.with_file_name(tmp_game_dir_name);
    let hash_path = tmp_game_dir.join(format!("{}.crc32", game.id));

    let (processor_threads, preloader_threads) = get_threads_num();
    let disc_opts = DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    };

    let process_opts = ProcessOptions {
        processor_threads,
        scrub: ScrubLevel::UpdatePartition,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: false,
        digest_xxh64: false,
    };

    let get_file_name = |i| match i {
        0 => format!("{}.wbfs", game.id),
        n => format!("{}.wbf{n}", game.id),
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

    fs::remove_dir_all(&game.path)?;
    fs::rename(tmp_game_dir, &game.path)?;

    Ok(())
}
