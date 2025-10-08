// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    TaskType,
    convert::get_disc_opts,
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
    tasks::TaskProcessor,
    util,
};
use anyhow::{Result, anyhow};
use nod::{
    read::DiscReader,
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use size::Size;
use slint::ToSharedString;
use std::{path::Path, sync::Arc};

pub fn verify_game(game_dir: &Path, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    let dir_name = game_dir
        .file_name()
        .ok_or(anyhow!("Failed to get game name"))?
        .to_str()
        .ok_or(anyhow!("Failed to get game name"))?
        .to_string();

    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let overflow = get_overflow_file(&path);

    let (_, processor_threads) = util::get_threads_num();

    task_processor.spawn(Box::new(move |weak| {
        let status = format!("Verifying {}...", &dir_name);
        weak.upgrade_in_event_loop(move |handle| {
            handle.set_status(status.to_shared_string());
            handle.set_task_type(TaskType::VerifyingGame);
        })?;

        let disc = if let Some(overflow) = overflow {
            let reader = OverflowReader::new(&path, &overflow)?;
            DiscReader::new_stream(Box::new(reader), &get_disc_opts())?
        } else {
            DiscReader::new(&path, &get_disc_opts())?
        };

        let original_xxh64 = disc.meta().xxh64;

        let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;

        let finalization = disc_writer.process(
            |_, progress, total| {
                let dir_name = dir_name.clone();
                let _ = weak.upgrade_in_event_loop(move |handle| {
                    let status = format!(
                        "Verifying {}... ({}/{})",
                        &dir_name,
                        Size::from_bytes(progress),
                        Size::from_bytes(total)
                    );
                    handle.set_status(status.to_shared_string());
                });

                Ok(())
            },
            &ProcessOptions {
                processor_threads,
                digest_crc32: false,
                digest_md5: false,
                digest_sha1: false,
                digest_xxh64: true,
                scrub_update_partition: false,
            },
        )?;

        if let Some(original_xxh64) = original_xxh64
            && let Some(xxh64) = finalization.xxh64
        {
            if original_xxh64 == xxh64 {
                Ok(format!("{} XXH64 matches!", &dir_name))
            } else {
                let msg = format!(
                    "{} XXH64 doesn't match!\nExpected: {:x}\nActual: {:x}\n\nThe game has been altered!\n\nThis can also happen if a game partition was removed",
                    &dir_name, original_xxh64, xxh64
                );

                Err(anyhow!(msg))
            }
        } else {
            Err(anyhow!("Didn't find XXH64 hashes"))
        }
    }))
}
