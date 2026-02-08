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
use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Seek, Write},
    path::PathBuf,
    thread,
};
use zip::ZipArchive;

pub const SPLIT_SIZE: usize = 4_294_934_528; // 4 GiB - 32 KiB (fits on a u32)

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
                        let out = File::create(&new_source_path)?;
                        let mut writer = BufWriter::new(out);
                        io::copy(&mut archived_disc, &mut writer)?;
                        writer.flush()?;
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
                let is_wii = disc_reader.header().is_wii();
                let title = disc_reader.header().game_title_str().to_string();
                let id = disc_reader.header().game_id_str().to_string();

                let out_format = if is_wii {
                    self.config.wii_output_format()
                } else {
                    self.config.gc_output_format()
                };

                let (parent, mut out_path) = if is_wii {
                    let title = util::sanitize(&title);

                    let parent = self
                        .config
                        .mount_point()
                        .join("wbfs")
                        .join(format!("{title} [{id}]"));

                    let out_path = parent.join(&id).with_extension(format_to_ext(out_format));

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

                let must_split = is_wii && (self.config.always_split() || self.is_fat32);

                if must_split && out_format == nod::common::Format::Iso {
                    out_path = out_path.with_extension("part0.iso");
                }

                if out_path.exists() {
                    return Ok(files_to_remove);
                }

                fs::create_dir_all(&parent)?;
                let out_file = File::create(out_path)?;
                let mut out_writer = BufWriter::new(out_file);
                let mut overflow_writer: Option<BufWriter<File>> = None;

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

                let mut prev_percentage = 100;
                let mut bytes_written_in_first_split = 0;
                let finalization = disc_writer.process(
                    |data, progress, total| {
                        if let Some(overflow_writer) = &mut overflow_writer {
                            overflow_writer.write_all(&data)?;
                        } else if must_split {
                            let remaining_in_first_split =
                                SPLIT_SIZE - bytes_written_in_first_split;

                            let bytes_to_write_count = data.len();

                            if remaining_in_first_split < bytes_to_write_count {
                                let overflow_path = if out_format == nod::common::Format::Wbfs {
                                    parent.join(&id).with_extension("wbf1")
                                } else {
                                    parent.join(&id).with_extension("part1.iso")
                                };

                                let overflow_file = File::create(overflow_path)?;
                                let overflow_writer =
                                    overflow_writer.insert(BufWriter::new(overflow_file));

                                #[allow(clippy::cast_possible_truncation)]
                                out_writer.write_all(&data[..remaining_in_first_split])?;
                                overflow_writer.write_all(&data[remaining_in_first_split..])?;

                                // we don't update bytes_written_in_first_split as we won't access
                                // it anymore; theoretically it should now be SPLIT_SIZE
                            } else {
                                out_writer.write_all(&data)?;
                                bytes_written_in_first_split += bytes_to_write_count;
                            }
                        } else {
                            out_writer.write_all(&data)?;
                        }

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
                if let Some(overflow_writer) = &mut overflow_writer {
                    overflow_writer.flush()?;
                }

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
