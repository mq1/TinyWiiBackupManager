// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::{
    disc_info::DiscInfo, extensions::format_to_opts, game::Game, util::get_threads_num,
};
use anyhow::Result;
use iced::{
    futures::{StreamExt, channel::mpsc},
    task::{Straw, sipper},
};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use std::{sync::Arc, thread};

#[derive(Debug, Clone)]
pub struct ChecksumOperation {
    source: Game,
    display_str: String,
}

impl ChecksumOperation {
    pub fn new(source: Game) -> Self {
        let display_str = format!("Checksum {}", source.title());

        Self {
            source,
            display_str,
        }
    }

    pub fn run(self) -> impl Straw<Option<String>, String, Arc<anyhow::Error>> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(1);

            let handle = thread::spawn(move || -> Result<Option<String>> {
                let disc_info = DiscInfo::try_from_game_dir(self.source.path())?;

                let (processor_threads, preloader_threads) = get_threads_num();
                let process_opts = ProcessOptions {
                    processor_threads,
                    digest_crc32: true,
                    digest_md5: false,
                    digest_sha1: false,
                    digest_xxh64: false,
                    scrub: ScrubLevel::None,
                };

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };
                let disc_reader = DiscReader::new(disc_info.disc_path(), &disc_opts)?;

                let out_opts = format_to_opts(Format::Iso);
                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

                let mut prev_percentage = 100;
                let game_title = self.source.title();
                let finalization = disc_writer.process(
                    |_data, progress, total| {
                        let progress_percentage = progress * 100 / total;

                        if progress_percentage != prev_percentage {
                            let _ = tx.try_send(format!(
                                "✓ Hashing {game_title}  {progress_percentage:02}%"
                            ));
                            prev_percentage = progress_percentage;
                        }

                        Ok(())
                    },
                    &process_opts,
                )?;

                let msg = if disc_info.crc32() == finalization.crc32 {
                    format!(
                        "Hash match for {game_title}! — Embedded CRC32 is = to the actual file CRC32"
                    )
                } else {
                    format!(
                        "Hash mismatch for {game_title} — Embedded CRC32 is ≠ to the actual file CRC32\nThis is expected if the Update partition has been removed"
                    )
                };

                Ok(Some(msg))
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            handle
                .join()
                .expect("Failed to checksum game")
                .map_err(Arc::new)
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
