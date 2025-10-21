// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::ArchiveFormat,
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
    tasks::BackgroundMessage,
};
use anyhow::{anyhow, bail};
use nod::{
    common::{Compression, Format},
    read::DiscReader,
    write::{DiscWriter, FormatOptions},
};
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::PathBuf,
};

pub fn spawn_archive_game_task(app: &App, game_dir: PathBuf, out_path: PathBuf) {
    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(BackgroundMessage::UpdateStatus(
            "📦 Archiving game...".to_string(),
        ))?;

        let archive_format = match out_path.extension() {
            Some(ext) => match ext.to_str() {
                Some("rvz") => ArchiveFormat::Rvz,
                Some("iso") => ArchiveFormat::Iso,
                _ => bail!("Unsupported archive format"),
            },
            None => bail!("Unsupported archive format"),
        };

        let format_opts = match archive_format {
            ArchiveFormat::Rvz => FormatOptions {
                format: Format::Rvz,
                compression: Compression::Zstandard(19),
                block_size: Format::Rvz.default_block_size(),
            },
            ArchiveFormat::Iso => FormatOptions {
                format: Format::Iso,
                compression: Compression::None,
                block_size: Format::Iso.default_block_size(),
            },
        };

        msg_sender.send(BackgroundMessage::SetArchiveFormat(archive_format))?;

        let path = get_main_file(&game_dir).ok_or(anyhow!("No disc found"))?;
        let overflow = get_overflow_file(&path);

        let process_opts = get_process_opts(false);

        msg_sender.send(BackgroundMessage::UpdateStatus(format!(
            "📦 Archiving {}...",
            path.display()
        )))?;

        let disc = if let Some(overflow) = overflow {
            let reader = OverflowReader::new(&path, &overflow)?;
            DiscReader::new_stream(Box::new(reader), &get_disc_opts())?
        } else {
            DiscReader::new(&path, &get_disc_opts())?
        };

        let mut output_file = BufWriter::new(File::create(&out_path)?);
        let writer = DiscWriter::new(disc, &format_opts)?;

        let finalization = writer.process(
            |data, progress, total| {
                output_file.write_all(&data)?;

                let _ = msg_sender.send(BackgroundMessage::UpdateStatus(format!(
                    "📦 Archiving {}  {:02.0}%",
                    path.display(),
                    progress as f32 / total as f32 * 100.0
                )));

                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            output_file.rewind()?;
            output_file.write_all(finalization.header.as_ref())?;
        }

        msg_sender.send(BackgroundMessage::NotifyInfo(format!(
            "📦 Archived {}",
            path.display()
        )))?;

        Ok(())
    });
}
