// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    drive_info::DriveInfo,
    game::Game,
    util::{HEADER_SIZE, SPLIT_SIZE, get_threads_num},
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
    time::{Duration, Instant},
};
use which_fs::FsKind;

pub fn perform(
    game: Game,
    config: Config,
    drive_info: DriveInfo,
    update_progress: &impl Fn(u8),
) -> Result<()> {
    let disc_path = game.get_disc_path().ok_or(anyhow!("No disc found"))?;
    let game_dir_name = game.path.file_name().ok_or(anyhow!("No file name"))?;
    let tmp_game_dir_name = format!("{} SCRUB", game_dir_name.to_string_lossy());
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
        digest_crc32: false,
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

    let disc_reader = DiscReader::new(&disc_path, &disc_opts)?;
    let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(Format::Wbfs))?;

    fs::create_dir_all(&tmp_game_dir)?;
    let mut out_writer = BufWriter::with_capacity(
        32_768,
        SplitWriter::create(&tmp_game_dir, get_file_name, split_size)?,
    );
    let mut hasher = Hasher::new();
    let mut head_buffer = Vec::with_capacity(HEADER_SIZE);

    let mut last_update = Instant::now();
    let finalization = disc_writer.process(
        |data, progress, total| {
            out_writer.write_all(&data)?;

            let remaining_in_head = HEADER_SIZE.saturating_sub(head_buffer.len());
            if remaining_in_head > 0 {
                let to_write = remaining_in_head.min(data.len());
                head_buffer.extend_from_slice(&data[..to_write]);
                hasher.update(&data[to_write..]);
            } else {
                hasher.update(&data);
            }

            if last_update.elapsed() > Duration::from_millis(100) {
                let current_percentage = progress * 100 / total;
                (update_progress)(current_percentage as u8);

                last_update = Instant::now();
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
