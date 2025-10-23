// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
    tasks::{BackgroundMessage, TaskProcessor},
    wiitdb::GameInfo,
};
use anyhow::{Result, anyhow};
use crossbeam_channel::Sender;
use nod::{
    read::{DiscMeta, DiscReader},
    write::{DiscWriter, FormatOptions},
};
use std::path::{Path, PathBuf};

pub fn spawn_checksum_task(
    game_dir: PathBuf,
    game_info: Option<GameInfo>,
    task_processor: &TaskProcessor,
) {
    task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "🔎 Performing game checksum...".to_string(),
        ))?;

        let embedded = get_embedded_hashes(&game_dir)?;
        let crc32 = calc_crc32(&game_dir, msg_sender)?;

        if let Some(embedded_crc32) = embedded.crc32
        {
            if embedded_crc32 == crc32 {
                msg_sender.send(BackgroundMessage::NotifySuccess(
                    "🔎 Embedded CRC32 is == to the actual file CRC32".to_string()
                ))?;
            } else {
                msg_sender.send(BackgroundMessage::NotifyError(
                    format!("🔎 Embedded CRC32 mismatch, expected: {:x}, found: {:x}\n\nThis is expected if the Update partition has been removed",
                    embedded_crc32,
                    crc32
                )))?;
            }
        }

        if let Some(game_info) = game_info {
            if game_info.roms.iter().filter_map(|r| r.crc).any(|db_crc32| db_crc32 == crc32) {
                msg_sender.send(BackgroundMessage::NotifySuccess(
                    "🔎 CRC32 matches the Redump hash: your dump is perfect!".to_string(),
                ))?;
            } else {
                msg_sender.send(BackgroundMessage::NotifyError(
                    "🔎 CRC32 does not match the Redump hash".to_string(),
                ))?;
            }
        }

        Ok(())
    });
}

fn get_embedded_hashes(game_dir: &Path) -> Result<DiscMeta> {
    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let disc = DiscReader::new(&path, &get_disc_opts())?;
    let meta = disc.meta();
    Ok(meta)
}

pub fn calc_crc32(game_dir: &Path, msg_sender: &Sender<BackgroundMessage>) -> Result<u32> {
    let game_dir_name = game_dir
        .file_name()
        .ok_or(anyhow!("Failed to get disc name"))?
        .to_str()
        .ok_or(anyhow!("Failed to get disc name"))?
        .to_string();

    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let overflow = get_overflow_file(&path);

    let disc = if let Some(overflow) = overflow {
        let reader = OverflowReader::new(&path, &overflow)?;
        DiscReader::new_stream(Box::new(reader), &get_disc_opts())?
    } else {
        DiscReader::new(&path, &get_disc_opts())?
    };

    let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;

    let finalization = disc_writer.process(
        |_, progress, total| {
            let _ = msg_sender.send(BackgroundMessage::UpdateStatus(format!(
                "🔎 Hashing {}  {:02.0}%",
                &game_dir_name,
                progress as f32 / total as f32 * 100.0,
            )));

            Ok(())
        },
        &get_process_opts(false),
    )?;

    let crc32 = finalization.crc32.ok_or(anyhow!("Failed to get CRC32"))?;

    Ok(crc32)
}
