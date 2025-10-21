// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
    tasks::{BackgroundMessage, TaskProcessor},
};
use anyhow::{Result, anyhow, bail};
use crossbeam_channel::Sender;
use nod::{
    read::{DiscMeta, DiscReader},
    write::{DiscFinalization, DiscWriter, FormatOptions},
};
use std::path::{Path, PathBuf};

pub fn spawn_verify_game_task(game_dir: PathBuf, task_processor: &TaskProcessor) {
    task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "🔎 Verifying game...".to_string(),
        ))?;

        let embedded = get_embedded_hashes(&game_dir)?;
        let finalization = calc_hashes(&game_dir, msg_sender)?;

        if let Some(crc32) = finalization.crc32
            && let Some(embedded_crc32) = embedded.crc32
            && crc32 != embedded_crc32
        {
            bail!(
                "CRC32 mismatch, expected: {:x}, found: {:x}\n\nThis is expected if the Update partition has been removed",
                embedded_crc32,
                crc32
            );
        }

        if let Some(md5) = finalization.md5
            && let Some(embedded_md5) = embedded.md5
            && md5 != embedded_md5
        {
            bail!(
                "MD5 mismatch, expected: {}, found: {}\n\nThis is expected if the Update partition has been removed",
                hex::encode(embedded_md5),
                hex::encode(md5)
            );
        }

        if let Some(sha1) = finalization.sha1
            && let Some(embedded_sha1) = embedded.sha1
            && sha1 != embedded_sha1
        {
            bail!(
                "SHA1 mismatch, expected: {}, found: {}\n\nThis is expected if the Update partition has been removed",
                hex::encode(embedded_sha1),
                hex::encode(sha1)
            );
        }

        if let Some(xxh64) = finalization.xxh64
            && let Some(embedded_xxh64) = embedded.xxh64
            && xxh64 != embedded_xxh64
        {
            bail!(
                "XXH64 mismatch, expected: {:x}, found: {:x}\n\nThis is expected if the Update partition has been removed",
                embedded_xxh64,
                xxh64
            );
        }

        msg_sender.send(BackgroundMessage::NotifyInfo(
            "🔎 Game successfully verified".to_string()
        ))?;

        Ok(())
    });
}

fn get_embedded_hashes(game_dir: &Path) -> Result<DiscMeta> {
    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let disc = DiscReader::new(&path, &get_disc_opts())?;
    let meta = disc.meta();
    Ok(meta)
}

pub fn calc_hashes(
    game_dir: &Path,
    msg_sender: &Sender<BackgroundMessage>,
) -> Result<DiscFinalization> {
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

    Ok(finalization)
}
