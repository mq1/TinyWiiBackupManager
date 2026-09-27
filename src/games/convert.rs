// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        conversion_state::ConversionState, disc_reader::get_disc_reader, export::ext_to_format,
    },
    util::misc::OPTIMAL_THREADS,
};
use anyhow::{Context, Result};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel};
use smol::stream::Stream;
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Seek, Write},
    path::PathBuf,
};

fn perform_blocking(
    disc_path: PathBuf,
    out_path: PathBuf,
    tx: &smol::channel::Sender<ConversionState>,
) -> Result<String> {
    let filename = out_path
        .file_name()
        .and_then(OsStr::to_str)
        .context("invalid filename")?;

    let ext = out_path
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ext_to_format)
        .context("invalid extension")?;

    let format_opts = FormatOptions::new(ext);

    let disc_reader = get_disc_reader(&disc_path)?;
    let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;
    let mut out_writer = {
        let file = File::create(&out_path)?;
        BufWriter::new(file)
    };

    let mut prev_percentage = 100;
    let finalization = disc_writer.process(
        |data, progress, total| {
            out_writer.write_all(&data)?;

            let progress_percentage = progress * 100 / total;
            if progress_percentage != prev_percentage {
                let _ = tx.try_send(ConversionState::Progress(format!(
                    "Converting into {filename}  {progress_percentage:02}%",
                )));

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
    Ok(filename.to_string())
}

pub fn convert_game(disc_path: PathBuf, out_path: PathBuf) -> impl Stream<Item = ConversionState> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = match perform_blocking(disc_path, out_path, &tx) {
            Ok(filename) => ConversionState::Finished(format!("Converted {filename}")),
            Err(e) => ConversionState::Errored(e.to_string()),
        };

        tx.send_blocking(exit).expect("Channel should be open");
    });

    rx
}
