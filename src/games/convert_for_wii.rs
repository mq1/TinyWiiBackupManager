// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::{
        extensions::{format_to_ext, format_to_opts},
        util::get_threads_num,
    },
    util,
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
use split_write::SplitWriter;
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Seek, Write},
    num::NonZeroU64,
    path::PathBuf,
    thread,
};
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroU64 = NonZeroU64::new(4 * 1024 * 1024 * 1024 - 32 * 1024).unwrap(); // 4 GiB - 32 KiB

#[derive(Debug, Clone)]
pub struct ConvertForWiiOperation {
    source_path: PathBuf,
    display_str: String,
    config: Config,
    is_fat32: bool,
}

impl ConvertForWiiOperation {
    pub fn new(source_path: PathBuf, config: Config, is_fat32: bool) -> Self {
        let display_str = format!("⤒ Convert {}", source_path.display());

        Self {
            source_path,
            display_str,
            config,
            is_fat32,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn run(mut self) -> impl Straw<Option<String>, String, String> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(1);

            let handle = thread::spawn(move || -> Result<Vec<PathBuf>> {
                let mut files_to_remove = Vec::new();
                if self.config.remove_sources_games() {
                    files_to_remove.push(self.source_path.clone());
                }

                if self
                    .source_path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
                {
                    let _ = tx.try_send(format!("Unzipping {}", self.source_path.display()));

                    let file = File::open(&self.source_path)?;
                    let reader = BufReader::new(file);
                    let mut archive = ZipArchive::new(reader)?;
                    let mut archived_disc = archive.by_index(0)?;

                    let Some(parent) = self.source_path.parent() else {
                        bail!("No parent dir found");
                    };

                    let new_source_path = parent.join(archived_disc.name());
                    if !new_source_path.exists() {
                        let mut out = File::create(&new_source_path)?;
                        io::copy(&mut archived_disc, &mut out)?;
                        out.flush()?;
                        files_to_remove.push(new_source_path.clone());
                    }

                    self.source_path = new_source_path;
                }

                let (processor_threads, preloader_threads) = get_threads_num();

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };

                let disc_reader = DiscReader::new(&self.source_path, &disc_opts)?;
                let header = disc_reader.header();
                let is_wii = header.is_wii();
                let title = header.game_title_str().to_string();
                let id = header.game_id_str().to_string();
                let disc_num = header.disc_num;

                let out_format = if is_wii {
                    self.config.wii_output_format()
                } else {
                    self.config.gc_output_format()
                };

                let parent_dir = if is_wii {
                    self.config
                        .mount_point()
                        .join("wbfs")
                        .join(format!("{title} [{id}]"))
                } else {
                    let title = util::sanitize(&title)
                        .replace(" game disc 1", "")
                        .replace(" game disc 2", "");

                    self.config
                        .mount_point()
                        .join("games")
                        .join(format!("{title} [{id}]"))
                };

                let must_split = is_wii && (self.config.always_split() || self.is_fat32);

                let get_file_name = |i| {
                    if is_wii {
                        let ext = if out_format == nod::common::Format::Wbfs {
                            match i {
                                0 => "wbfs".to_string(),
                                n => format!("wbf{n}"),
                            }
                        } else if must_split {
                            format!("part{i}.iso")
                        } else {
                            "iso".to_string()
                        };

                        format!("{id}.{ext}")
                    } else {
                        let file_stem = match disc_num {
                            0 => "game".to_string(),
                            n => format!("disc{n}"),
                        };

                        let ext = format_to_ext(out_format);
                        format!("{file_stem}.{ext}")
                    }
                };

                if parent_dir.join(get_file_name(0)).exists() {
                    return Ok(files_to_remove);
                }

                fs::create_dir_all(&parent_dir)?;

                let out_opts = format_to_opts(out_format);
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

                let split_size = if must_split {
                    SPLIT_SIZE
                } else {
                    NonZeroU64::MAX
                };

                let mut out_writer = BufWriter::with_capacity(
                    32_768,
                    SplitWriter::new(parent_dir, get_file_name, split_size),
                );

                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

                let mut prev_percentage = 100;
                let finalization = disc_writer.process(
                    |data, progress, total| {
                        out_writer.write_all(&data)?;

                        let progress_percentage = progress * 100 / total;
                        if progress_percentage != prev_percentage {
                            let _ = tx.try_send(format!(
                                "⤒ Converting {title}  {progress_percentage:02}%"
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
                Ok(files_to_remove)
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            let files_to_remove = handle
                .join()
                .expect("Failed to join thread")
                .map_err(|e| format!("Failed to convert game: {e:#}"))?;

            for path in files_to_remove {
                let _ = fs::remove_file(path);
            }

            Ok(None)
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
