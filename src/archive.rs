// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{
    config::ArchiveFormat,
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_overflow_file},
};
use anyhow::{Result, bail};
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

pub fn spawn_archive_game_task(
    app: &App,
    path: PathBuf,
    out_path: PathBuf,
) -> Result<ArchiveFormat> {
    let archive_format = match out_path.extension() {
        Some(ext) => match ext.to_str() {
            Some("rvz") => ArchiveFormat::Rvz,
            Some("iso") => ArchiveFormat::Iso,
            _ => bail!(
                "{} Unsupported archive format",
                egui_phosphor::regular::FILE_ARCHIVE
            ),
        },
        None => bail!(
            "{} Unsupported archive format",
            egui_phosphor::regular::FILE_ARCHIVE
        ),
    };

    app.task_processor.spawn(move |msg_sender| {
        msg_sender.send(Message::UpdateStatus(format!(
            "{} Archiving game...",
            egui_phosphor::regular::FILE_ARCHIVE
        )))?;

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

        let overflow = get_overflow_file(&path);

        let process_opts = get_process_opts(false);

        msg_sender.send(Message::UpdateStatus(format!(
            "{} Archiving {}...",
            egui_phosphor::regular::FILE_ARCHIVE,
            path.display()
        )))?;

        let disc = if let Some(overflow) = overflow {
            let reader = OverflowReader::new(&path, &overflow)?;
            DiscReader::new_from_non_cloneable_read(reader, &get_disc_opts())?
        } else {
            DiscReader::new(&path, &get_disc_opts())?
        };

        let mut output_file = BufWriter::new(File::create(&out_path)?);
        let writer = DiscWriter::new(disc, &format_opts)?;

        let finalization = writer.process(
            |data, progress, total| {
                output_file.write_all(&data)?;

                let _ = msg_sender.send(Message::UpdateStatus(format!(
                    "{} Archiving {}  {:02.0}%",
                    egui_phosphor::regular::FILE_ARCHIVE,
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

        msg_sender.send(Message::NotifyInfo(format!(
            "{} Archived {}",
            egui_phosphor::regular::FILE_ARCHIVE,
            path.display()
        )))?;

        Ok(())
    });

    Ok(archive_format)
}
