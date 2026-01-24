// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::extensions::{format_to_ext, format_to_opts},
    util,
};
use anyhow::Result;
use iced::{
    futures::{StreamExt, channel::mpsc},
    task::{Straw, sipper},
};
use nod::{
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
    path::PathBuf,
    sync::Arc,
    thread,
};

#[derive(Debug, Clone)]
pub struct ConvertForWiiOperation {
    source_path: PathBuf,
    display_str: String,
    config: Config,
}

impl ConvertForWiiOperation {
    pub fn new(source_path: PathBuf, config: Config) -> Self {
        let display_str = format!("⤒ Convert {}", source_path.display());

        Self {
            source_path,
            display_str,
            config,
        }
    }

    pub fn run(self) -> impl Straw<String, String, Arc<anyhow::Error>> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(100);

            let handle = thread::spawn(move || -> Result<String> {
                let (processor_threads, preloader_threads) = get_threads_num();

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };

                let disc_reader = DiscReader::new(&self.source_path, &disc_opts)?;
                let is_wii = disc_reader.header().is_wii();
                let title = disc_reader.header().game_title_str().to_string();
                let id = disc_reader.header().game_id_str();

                let out_format = if is_wii {
                    self.config.wii_output_format()
                } else {
                    self.config.gc_output_format()
                };

                let (parent, out_path) = if is_wii {
                    let title = util::sanitize(&title);

                    let parent = self
                        .config
                        .mount_point()
                        .join("wbfs")
                        .join(format!("{title} [{id}]"));

                    let out_path = parent.join(id).with_extension(format_to_ext(out_format));

                    (parent, out_path)
                } else {
                    let disc_name = match disc_reader.header().disc_num {
                        0 => "game".to_string(),
                        n => format!("game{}", n + 1),
                    };

                    let title = util::sanitize(&title)
                        .replace(" game disc 1", "")
                        .replace(" game disc 2", "");

                    let parent = self
                        .config
                        .mount_point()
                        .join("games")
                        .join(format!("{title} [{id}]"));

                    let out_path = parent
                        .join(disc_name)
                        .with_extension(format_to_ext(out_format));

                    (parent, out_path)
                };

                fs::create_dir_all(parent)?;
                let out_file = File::create(out_path)?;
                let mut out_writer = BufWriter::new(out_file);

                let out_opts = format_to_opts(out_format);
                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

                let process_opts = ProcessOptions {
                    processor_threads,
                    digest_crc32: true,
                    digest_md5: false,
                    digest_sha1: true,
                    digest_xxh64: true,
                    scrub: if self.config.scrub_update_partition() {
                        ScrubLevel::UpdatePartition
                    } else {
                        ScrubLevel::None
                    },
                };

                let finalization = disc_writer.process(
                    |data, progress, total| {
                        out_writer.write_all(&data)?;

                        let _ = tx.try_send(format!(
                            "⤒ Converting {}  {:02}%",
                            title,
                            progress * 100 / total
                        ));

                        Ok(())
                    },
                    &process_opts,
                )?;

                if !finalization.header.is_empty() {
                    out_writer.rewind()?;
                    out_writer.write_all(&finalization.header)?;
                }

                out_writer.flush()?;

                if self.config.remove_sources_games() {
                    fs::remove_file(self.source_path)?;
                }

                let msg = format!("Converted {title}");
                Ok(msg)
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            handle
                .join()
                .expect("Failed to convert game")
                .map_err(Arc::new)
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}

pub fn get_threads_num() -> (usize, usize) {
    let cpus = num_cpus::get();

    let preloader_threads = match cpus {
        0..=4 => 1,
        5..=8 => 2,
        _ => 4,
    };

    let processor_threads = cpus - preloader_threads;

    (preloader_threads, processor_threads)
}
