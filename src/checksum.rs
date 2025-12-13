// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::disc_info::DiscInfo;
use crate::messages::Message;
use crate::{
    convert::{get_disc_opts, get_process_opts},
    wiitdb::GameInfo,
};
use anyhow::{Result, anyhow};
use crossbeam_channel::Sender;
use egui_phosphor::bold as ph;
use nod::{
    read::DiscReader,
    write::{DiscWriter, FormatOptions},
};

pub fn spawn_checksum_task(app: &App, disc_info: DiscInfo, game_info: Option<&GameInfo>) {
    let redump_crc32s = game_info
        .iter()
        .flat_map(|g| &g.roms)
        .flat_map(|r| r.crc)
        .collect::<Box<[_]>>();

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(
            format!("{} Performing game checksum...", ph::FINGERPRINT_SIMPLE),
        ))?;

        let crc32 = calc_crc32(&disc_info, msg_sender)?;

        if let Some(embedded_crc32) = disc_info.crc32 {
            if embedded_crc32 == crc32 {
                msg_sender.send(Message::NotifySuccess(
                    format!("{} Embedded CRC32 is == to the actual file CRC32", ph::FINGERPRINT_SIMPLE),
                ))?;
            } else {
                let e = anyhow!(
                    "{} Embedded CRC32 mismatch, expected: {:x}, found: {:x}\n\nThis is expected if the Update partition has been removed",
                    ph::FINGERPRINT_SIMPLE, embedded_crc32, crc32
                );

                msg_sender.send(Message::NotifyError(e))?;
            }
        }

        if !redump_crc32s.is_empty() {
            if redump_crc32s.contains(&crc32) {
                msg_sender.send(Message::NotifySuccess(
                    format!("{} CRC32 matches the Redump hash: your dump is perfect!", ph::FINGERPRINT_SIMPLE),
                ))?;
            } else {
                let e = anyhow!(
                    "{} CRC32 does not match the Redump hash", ph::FINGERPRINT_SIMPLE
                );

                msg_sender.send(Message::NotifyError(e))?;
            }
        }

        Ok(())
    });
}

pub fn calc_crc32(disc_info: &DiscInfo, msg_sender: &Sender<Message>) -> Result<u32> {
    let disc = DiscReader::new(&disc_info.disc_path, &get_disc_opts())?;
    let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;

    let finalization = disc_writer.process(
        |_, progress, total| {
            let _ = msg_sender.send(Message::UpdateStatus(format!(
                "{} Hashing {}  {:02}%",
                ph::FINGERPRINT_SIMPLE,
                &disc_info.title,
                progress * 100 / total,
            )));

            Ok(())
        },
        &get_process_opts(false),
    )?;

    let crc32 = finalization
        .crc32
        .ok_or(anyhow!("{} Failed to get CRC32", ph::FINGERPRINT_SIMPLE))?;

    Ok(crc32)
}
