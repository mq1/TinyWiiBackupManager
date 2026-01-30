// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::{
    extensions::{ext_to_format, format_to_opts},
    util::get_threads_num,
};
use anyhow::{Result, bail};
use iced::{
    futures::{StreamExt, channel::mpsc},
    task::{Straw, sipper},
};
use nod::{
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::PathBuf,
    thread,
};

#[derive(Debug, Clone)]
pub struct ArchiveOperation {
    source: PathBuf,
    title: String,
    dest: PathBuf,
    display_str: String,
}

impl ArchiveOperation {
    pub fn new(source: PathBuf, title: String, dest: PathBuf) -> Self {
        let display_str = format!("⤓ Archive {title}");

        Self {
            source,
            title,
            dest,
            display_str,
        }
    }

    pub fn run(self) -> impl Straw<Option<String>, String, String> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(1);

            let handle = thread::spawn(move || -> Result<()> {
                let Some(out_format) = ext_to_format(self.dest.extension()) else {
                    bail!("Unsupported extension");
                };

                let (processor_threads, preloader_threads) = get_threads_num();
                let process_opts = ProcessOptions {
                    processor_threads,
                    digest_crc32: true,
                    digest_md5: false,
                    digest_sha1: true,
                    digest_xxh64: true,
                    scrub: ScrubLevel::None,
                };

                let out_file = File::create(&self.dest)?;
                let mut out_writer = BufWriter::new(out_file);

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };
                let disc_reader = DiscReader::new(&self.source, &disc_opts)?;

                let out_opts = format_to_opts(out_format);
                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

                let mut prev_percentage = 100;
                let finalization = disc_writer.process(
                    |data, progress, total| {
                        out_writer.write_all(&data)?;

                        let progress_percentage = progress * 100 / total;
                        if progress_percentage != prev_percentage {
                            let _ = tx.try_send(format!(
                                "⤓ Archiving {}  {:02}%",
                                self.title, progress_percentage
                            ));
                            prev_percentage = progress_percentage;
                        }

                        Ok(())
                    },
                    &process_opts,
                )?;

                if !finalization.header.is_empty() {
                    out_writer.rewind()?;
                    out_writer.write_all(&finalization.header)?;
                }

                out_writer.flush()?;

                Ok(())
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            handle
                .join()
                .expect("Failed to join thread")
                .map(|()| None)
                .map_err(|e| format!("Failed to archive game: {e:#}"))
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
