// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    MainWindow, TaskType,
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
};
use anyhow::{Result, anyhow, bail};
use nod::{
    common::Format,
    read::{DiscMeta, DiscReader},
    write::{DiscFinalization, DiscWriter, FormatOptions},
};
use size::Size;
use slint::{ToSharedString, Weak};
use std::path::{Path, PathBuf};

pub const NKIT_ADDR: u64 = 0x10000;
pub const NKIT_LEN: usize = 68;

pub fn verify_game(game_dir_str: &str, weak: &Weak<MainWindow>) -> Result<()> {
    let game_dir = PathBuf::from(game_dir_str);

    weak.upgrade_in_event_loop(move |handle| {
        handle.set_task_type(TaskType::VerifyingGame);
    })?;

    let embedded = get_embedded_hashes(&game_dir)?;
    let finalization = calc_hashes(&game_dir, weak)?.0;

    if let Some(crc32) = finalization.crc32
        && let Some(embedded_crc32) = embedded.crc32
        && crc32 != embedded_crc32 {
            bail!("CRC32 mismatch");
        }

    if let Some(md5) = finalization.md5
        && let Some(embedded_md5) = embedded.md5
        && md5 != embedded_md5 {
            bail!("MD5 mismatch");
        }

    if let Some(sha1) = finalization.sha1
        && let Some(embedded_sha1) = embedded.sha1
        && sha1 != embedded_sha1 {
            bail!("SHA1 mismatch");
        }

    if let Some(xxh64) = finalization.xxh64
        && let Some(embedded_xxh64) = embedded.xxh64
        && xxh64 != embedded_xxh64 {
            bail!("XXH64 mismatch");
        }

    Ok(())
}

fn get_embedded_hashes(game_dir: &Path) -> Result<DiscMeta> {
    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let disc = DiscReader::new(&path, &get_disc_opts())?;
    let meta = disc.meta();
    Ok(meta)
}

// Also returns the nkit header bytes
pub fn calc_hashes(
    game_dir: &Path,
    weak: &Weak<MainWindow>,
) -> Result<(DiscFinalization, Box<[u8; 68]>)> {
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

    let disc_writer = DiscWriter::new(disc, &FormatOptions::new(Format::Wbfs))?;

    let mut nkit_header = Box::new([0u8; NKIT_LEN]);

    let finalization = disc_writer.process(
        |bytes, progress, total| {
            let status = format!(
                "Hashing {}... ({}/{})",
                &game_dir_name,
                Size::from_bytes(progress),
                Size::from_bytes(total)
            );

            let _ = weak.upgrade_in_event_loop(move |handle| {
                handle.set_status(status.to_shared_string());
            });

            // check if we're at the nkit header
            if progress == NKIT_ADDR {
                let len = bytes.len().min(NKIT_LEN);
                nkit_header[..len].copy_from_slice(&bytes[..len]);
            }

            Ok(())
        },
        &get_process_opts(false),
    )?;

    Ok((finalization, nkit_header))
}
