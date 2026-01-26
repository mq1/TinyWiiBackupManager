// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        convert_for_wii::SPLIT_SIZE, disc_info::DiscInfo, extensions::format_to_opts, game::Game,
        util::get_threads_num,
    },
    util,
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
use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
    path::PathBuf,
    sync::Arc,
    thread,
};

#[derive(Debug, Clone)]
pub struct StripOperation {
    source: Game,
    display_str: String,
    always_split: bool,
}

impl StripOperation {
    pub fn new(source: Game, always_split: bool) -> Self {
        let display_str = format!("Remove update partition from {}", source.title());

        Self {
            source,
            display_str,
            always_split,
        }
    }

    pub fn run(self) -> impl Straw<Option<String>, String, Arc<anyhow::Error>> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(1);

            let handle = thread::spawn(move || -> Result<Option<String>> {
                let disc_info = DiscInfo::try_from_game_dir(self.source.path())?;
                if !disc_info.is_worth_stripping() {
                    return Ok(None);
                }

                let (processor_threads, preloader_threads) = get_threads_num();
                let process_opts = ProcessOptions {
                    processor_threads,
                    digest_crc32: true,
                    digest_md5: false,
                    digest_sha1: true,
                    digest_xxh64: true,
                    scrub: ScrubLevel::UpdatePartition,
                };

                let must_split = self.always_split || !util::can_write_over_4gb(self.source.path());

                let out_path = disc_info.disc_path().with_extension("wbfs.new");
                let out_file = File::create(&out_path)?;
                let mut out_writer = BufWriter::new(out_file);
                let mut overflow_path: Option<PathBuf> = None;
                let mut overflow_writer: Option<BufWriter<File>> = None;

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };
                let disc_reader = DiscReader::new(disc_info.disc_path(), &disc_opts)?;

                let out_opts = format_to_opts(Format::Wbfs);
                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

                let mut prev_percentage = 100;
                let mut bytes_written_in_first_split = 0;
                let game_title = self.source.title();
                let finalization = disc_writer.process(
                    |data, progress, total| {
                        if let Some(overflow_writer) = &mut overflow_writer {
                            overflow_writer.write_all(&data)?;
                        } else if must_split {
                            let remaining_in_first_split =
                                SPLIT_SIZE - bytes_written_in_first_split;

                            let bytes_to_write_count = data.len();

                            if remaining_in_first_split < bytes_to_write_count {
                                let overflow_path = overflow_path.insert(disc_info.disc_path().with_extension("wbf1.new"));
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
                                "Remove update partition from {game_title}  {progress_percentage:02}%"
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

                fs::rename(out_path, disc_info.disc_path())?;
                if let Some(overflow_path) = overflow_path.take() {
                    fs::rename(overflow_path, disc_info.disc_path().with_extension("wbf1"))?;
                }

                Ok(Some(format!("Removed update partition from {game_title}")))
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            handle
                .join()
                .expect("Failed to remove update partion")
                .map_err(Arc::new)
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
