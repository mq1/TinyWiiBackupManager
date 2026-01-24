// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::{
    disc_info,
    extensions::{ext_to_format, format_to_opts},
    game::Game,
};
use anyhow::bail;
use iced::{
    futures::executor::block_on,
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
};

#[derive(Debug, Clone)]
pub struct ArchiveOperation {
    source: Game,
    dest: PathBuf,
    display_str: String,
}

impl ArchiveOperation {
    pub fn new(source: Game, dest: PathBuf) -> Self {
        let display_str = format!("⤓ Archive {}", source.title());

        Self {
            source,
            dest,
            display_str,
        }
    }

    pub fn run(self) -> impl Straw<String, String, anyhow::Error> {
        sipper(async move |mut sender| {
            let disc_path = disc_info::get_main_disc_file_in_dir(self.source.path())?;

            let Some(out_format) = ext_to_format(self.dest.extension()) else {
                bail!("Unsupported extension");
            };

            let out_file = File::create(&self.dest)?;
            let mut out_writer = BufWriter::new(out_file);

            let disc_opts = DiscOptions {
                partition_encryption: PartitionEncryption::Original,
                preloader_threads: 1,
            };
            let disc_reader = DiscReader::new(disc_path, &disc_opts)?;

            let out_opts = format_to_opts(out_format);
            let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

            let process_opts = ProcessOptions {
                processor_threads: (num_cpus::get() - 1).max(1),
                digest_crc32: true,
                digest_md5: false,
                digest_sha1: true,
                digest_xxh64: true,
                scrub: ScrubLevel::None,
            };

            let finalization = disc_writer.process(
                |data, progress, total| {
                    out_writer.write_all(&data)?;

                    block_on(sender.send(format!(
                        "Archiving {}  {:02}%",
                        self.source.title(),
                        progress * 100 / total
                    )));

                    Ok(())
                },
                &process_opts,
            )?;

            if !finalization.header.is_empty() {
                out_writer.rewind()?;
                out_writer.write_all(&finalization.header)?;
            }

            out_writer.flush()?;

            let msg = format!("Archived {}", self.source.title());
            Ok(msg)
        })
    }

    pub fn display_str(&self) -> &str {
        &self.display_str
    }
}
