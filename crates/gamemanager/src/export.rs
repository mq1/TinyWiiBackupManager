// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_read::SplitFileReader;
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::Path,
};

use crate::{
    FromExt, OPTIMAL_THREADS, disc_stream::SplitFileDiscStream, game::Game,
    multi_disc::open_multi_disc_files,
};

pub fn export_game(game: &Game, out_path: &Path, send_progress: impl Fn(String)) -> Result<String> {
    let disc_reader = {
        let disc_opts = DiscOptions {
            preloader_threads: OPTIMAL_THREADS.preloader,
            ..Default::default()
        };

        let disc_path = game.get_disc_path().context("disc not found")?;
        let disc_files = open_multi_disc_files(&disc_path)?;
        let disc_files_reader = SplitFileReader::new(disc_files)?;
        DiscReader::new_stream(Box::new(SplitFileDiscStream(disc_files_reader)), &disc_opts)?
    };

    let ext = out_path
        .extension()
        .and_then(Format::from_ext)
        .context("invalid extension")?;

    let format_opts = FormatOptions::new(ext);

    let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;
    let mut out_writer = {
        let file = File::create(out_path)?;
        BufWriter::new(file)
    };

    let mut prev_percentage = 100;
    let finalization = disc_writer.process(
        |data, progress, total| {
            out_writer.write_all(&data)?;

            let progress_percentage = progress * 100 / total;
            if progress_percentage != prev_percentage {
                send_progress(format!(
                    "⤓  Exporting {}  {progress_percentage:02}%",
                    game.title()
                ));

                prev_percentage = progress_percentage;
            }

            Ok(())
        },
        &ProcessOptions {
            processor_threads: OPTIMAL_THREADS.processor,
            scrub: ScrubLevel::None,
            digest_crc32: true,
            digest_md5: false,
            digest_sha1: true,
            digest_xxh64: true,
        },
    )?;

    if !finalization.header.is_empty() {
        out_writer.rewind()?;
        out_writer.write_all(&finalization.header)?;
    }

    out_writer.flush()?;

    Ok(format!("Exported {}", game.title()))
}
