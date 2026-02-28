// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::{
    convert_for_wii::SPLIT_SIZE, disc_info::DiscInfo, extensions::format_to_opts, game::Game,
    util::get_threads_num,
};
use anyhow::{Result, anyhow};
use iced::{
    futures::{StreamExt, channel::mpsc},
    task::{Straw, sipper},
};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use split_write::SplitWriter;
use std::{
    fs,
    io::{BufWriter, Seek, Write},
    num::NonZeroU64,
    thread,
};

#[derive(Debug, Clone)]
pub struct StripOperation {
    source: Game,
    display_str: String,
    always_split: bool,
    is_fat32: bool,
}

impl StripOperation {
    pub fn new(source: Game, always_split: bool, is_fat32: bool) -> Self {
        let display_str = format!("Remove update partition from {}", source.title());

        Self {
            source,
            display_str,
            always_split,
            is_fat32,
        }
    }

    pub fn run(self) -> impl Straw<Option<String>, String, String> {
        sipper(async move |mut sender| {
            let (mut tx, mut rx) = mpsc::channel(1);

            let handle = thread::spawn(move || -> Result<()> {
                let disc_info = DiscInfo::try_from_game_dir(self.source.path())?;
                if !disc_info.is_worth_stripping() {
                    return Ok(());
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

                let must_split = self.always_split || self.is_fat32;

                let get_file_name = |i| {
                    let ext = match i {
                        0 => "wbfs".to_string(),
                        n => format!("wbf{n}"),
                    };

                    format!("{}.{ext}.new", self.source.id().as_str())
                };

                let split_size = if must_split {
                    SPLIT_SIZE
                } else {
                    NonZeroU64::MAX
                };

                let disc_opts = DiscOptions {
                    partition_encryption: PartitionEncryption::Original,
                    preloader_threads,
                };
                let disc_reader = DiscReader::new(disc_info.disc_path(), &disc_opts)?;

                let out_opts = format_to_opts(Format::Wbfs);
                let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;
                let dest_dir = self.source.path().clone();

                let file_count = {
                    let mut out_writer = BufWriter::with_capacity(
                        32_768,
                        SplitWriter::new(dest_dir, get_file_name, split_size),
                    );

                    let mut prev_percentage = 100;
                    let finalization = disc_writer.process(
                        |data, progress, total| {
                            out_writer.write_all(&data)?;

                            let progress_percentage = progress * 100 / total;
                            if progress_percentage != prev_percentage {
                                let _ = tx.try_send(format!(
                                    "Remove update partition from {}  {:02}%",
                                    self.source.title(),
                                    progress_percentage
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
                    out_writer
                        .into_inner()
                        .map_err(|_| anyhow!("Failed to get inner split writer"))?
                        .file_count()
                };

                for i in 0..file_count {
                    let file_name = get_file_name(i);
                    let original_file_name = file_name.strip_suffix(".new").unwrap().to_string();
                    let out_path = self.source.path().join(file_name);
                    let original_path = self.source.path().join(original_file_name);

                    let _ = fs::remove_file(&original_path);
                    fs::rename(&out_path, &original_path)?;
                }

                Ok(())
            });

            while let Some(msg) = rx.next().await {
                sender.send(msg).await;
            }

            handle
                .join()
                .expect("Failed to join thread")
                .map(|()| None)
                .map_err(|e| format!("Failed to remove update partion: {e:#}"))
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
