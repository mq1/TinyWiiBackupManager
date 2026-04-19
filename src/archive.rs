// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, QueuedArchiveConversion,
    extensions::{ext_to_format, format_to_opts},
    util::get_threads_num,
};
use anyhow::{Result, anyhow};
use nod::{
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use slint::{ToSharedString, Weak};
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub struct ArchiveConversion {
    pub game_title: String,
    pub in_path: PathBuf,
    pub out_path: PathBuf,
}

impl ArchiveConversion {
    pub fn new(queued: &QueuedArchiveConversion) -> Self {
        Self {
            game_title: queued.game_title.to_string(),
            in_path: PathBuf::from(&queued.in_path),
            out_path: PathBuf::from(&queued.out_path),
        }
    }

    pub fn perform(&self, weak: &Weak<AppWindow>) -> Result<()> {
        let in_path = Path::new(&self.in_path);
        let out_path = Path::new(&self.out_path);
        let out_ext = out_path.extension().ok_or(anyhow!("No extension"))?;
        let out_format = ext_to_format(out_ext).ok_or(anyhow!("Invalid extension"))?;

        let (processor_threads, preloader_threads) = get_threads_num();
        let disc_opts = DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        };

        let format_opts = format_to_opts(out_format);

        let process_opts = ProcessOptions {
            processor_threads,
            scrub: ScrubLevel::None,
            digest_crc32: true,
            digest_md5: false,
            digest_sha1: true,
            digest_xxh64: true,
        };

        let disc_reader = DiscReader::new(in_path, &disc_opts)?;
        let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;

        let mut out_writer = BufWriter::with_capacity(32_768, File::create(out_path)?);

        let status = format!("⤓  Archiving {}  (starting)", &self.game_title);
        let _ = weak.upgrade_in_event_loop(move |app| {
            app.set_status(status.to_shared_string());
        });

        let mut last_update = Instant::now();
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                if last_update.elapsed() > Duration::from_millis(200) {
                    let current_percentage = progress * 100 / total;

                    let status = format!(
                        "⤓  Archiving {}  {:02}%",
                        &self.game_title, current_percentage
                    );
                    let _ = weak.upgrade_in_event_loop(move |state| {
                        state.set_status(status.to_shared_string());
                    });

                    last_update = Instant::now();
                }

                Ok(())
            },
            &process_opts,
        )?;

        let mut out_file = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner split writer"))?;

        if !finalization.header.is_empty() {
            out_file.rewind()?;
            out_file.write_all(&finalization.header)?;
        }

        out_file.flush()?;

        Ok(())
    }
}
