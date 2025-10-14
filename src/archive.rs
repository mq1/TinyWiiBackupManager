// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ArchiveFormat, Config, MainWindow, TaskType,
    convert::{get_disc_opts, get_process_opts},
    overflow_reader::{OverflowReader, get_main_file, get_overflow_file},
};
use anyhow::{Result, anyhow};
use nod::{
    common::{Compression, Format},
    read::DiscReader,
    write::{DiscWriter, FormatOptions},
};
use rfd::FileDialog;
use slint::{ToSharedString, Weak};
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::Path,
};

pub fn archive_game(game_dir: &str, config: &Config, weak: &Weak<MainWindow>) -> Result<()> {
    let game_dir = Path::new(game_dir);
    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;

    let dest_dir = FileDialog::new()
        .set_title("Select destination directory")
        .pick_folder()
        .ok_or(anyhow!("No destination directory selected"))?;

    let base_name = game_dir
        .file_name()
        .ok_or(anyhow!("Failed to get disc name"))?
        .to_str()
        .ok_or(anyhow!("Failed to get disc name"))?
        .to_string();

    // Look for file overflows
    let overflow = get_overflow_file(&path);

    let out_path = dest_dir
        .join(&base_name)
        .with_extension(config.archive_format.extension());

    let process_opts = get_process_opts(false);

    let format_opts = match config.archive_format {
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

    let status = format!("Archiving {}...", path.display());
    weak.upgrade_in_event_loop(move |handle| {
        handle.set_status(status.to_shared_string());
        handle.set_task_type(TaskType::Archiving);
    })?;

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

            let status = format!(
                "Archiving {}  {:02.0}%",
                base_name,
                progress as f32 / total as f32 * 100.0
            );

            let _ = weak.upgrade_in_event_loop(move |handle| {
                handle.set_status(status.to_shared_string());
            });

            Ok(())
        },
        &process_opts,
    )?;

    if !finalization.header.is_empty() {
        output_file.rewind()?;
        output_file.write_all(finalization.header.as_ref())?;
    }

    Ok(())
}
