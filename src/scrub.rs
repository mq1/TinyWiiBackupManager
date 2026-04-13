// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConfigContents, DriveInfo, QueuedScrubConversion, State,
    convert::{HEADER_SIZE, SPLIT_SIZE},
    util::{get_disc_path, get_threads_num},
};
use anyhow::{Result, anyhow};
use crc32fast::Hasher;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use slint::{ToSharedString, Weak};
use split_write::SplitWriter;
use std::{
    fs,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct ScrubConversion {
    pub game_title: String,
    pub game_id: String,
    pub game_dir: PathBuf,
    pub always_split: bool,
    pub is_fat32: bool,
}

impl ScrubConversion {
    pub fn new(
        queued: &QueuedScrubConversion,
        conf: &ConfigContents,
        drive_info: &DriveInfo,
    ) -> Self {
        Self {
            game_title: queued.game.title.to_string(),
            game_id: queued.game.id.to_string(),
            game_dir: PathBuf::from(&queued.game.path),
            always_split: conf.always_split,
            is_fat32: drive_info.fs_kind == "FAT32",
        }
    }

    pub fn perform(&self, weak: &Weak<State<'static>>) -> Result<()> {
        let disc_path = get_disc_path(&self.game_dir)?;
        let game_dir_name = self.game_dir.file_name().ok_or(anyhow!("No file name"))?;
        let tmp_game_dir_name = format!("{} SCRUB", game_dir_name.to_string_lossy());
        let tmp_game_dir = self.game_dir.with_file_name(tmp_game_dir_name);
        let hash_path = tmp_game_dir.join(format!("{}.crc32", &self.game_id));

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
            0 => format!("{}.wbfs", &self.game_id),
            n => format!("{}.wbf{n}", &self.game_id),
        };

        let should_split = self.always_split || self.is_fat32;
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

        let status = format!("Scrubbing {}  (starting)", &self.game_title);
        let _ = weak.upgrade_in_event_loop(move |state| {
            state.set_status(status.to_shared_string());
        });

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

                if last_update.elapsed() > Duration::from_millis(200) {
                    let current_percentage = progress * 100 / total;

                    let status =
                        format!("Scrubbing {}  {:02}%", &self.game_title, current_percentage);
                    let _ = weak.upgrade_in_event_loop(move |state| {
                        state.set_status(status.to_shared_string());
                    });

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

        let mut final_hasher = Hasher::new();
        final_hasher.update(&head_buffer);
        final_hasher.combine(&hasher);
        let checksum = final_hasher.finalize();
        fs::write(hash_path, format!("{checksum:08x}"))?;

        fs::remove_dir_all(&self.game_dir)?;
        fs::rename(tmp_game_dir, &self.game_dir)?;

        Ok(())
    }
}

// use this only on wbfs files
pub fn is_worth_scrubbing<R: Read + Seek>(disc_reader: &mut R) -> Result<bool> {
    let mut buf = [0u8; 4];

    // check if the first partition is an update one
    disc_reader.seek(SeekFrom::Start(0x240024))?;
    disc_reader.read(&mut buf)?;
    if buf != [0, 0, 0, 1] {
        return Ok(false);
    }

    // check if the update data is unmapped
    disc_reader.seek(SeekFrom::Start(0x302))?;
    disc_reader.read(&mut buf)?;
    let worth_it = buf != [0, 0, 0, 0];

    Ok(worth_it)
}
