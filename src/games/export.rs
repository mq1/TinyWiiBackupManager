// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    errors::Error,
    games::{disc_reader::get_disc_reader, game::Game},
    util::misc::OPTIMAL_THREADS,
};
use nod::{
    common::Format,
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use sipper::{Straw, sipper};
use std::{
    ffi::OsStr,
    fs::File,
    io::{BufWriter, Seek, Write},
    path::PathBuf,
};
use tap::Pipe;

fn ext_to_format(ext: &str) -> Option<Format> {
    match ext {
        "iso" | "ISO" => Some(Format::Iso),
        "ciso" | "CISO" => Some(Format::Ciso),
        "gcz" | "CGZ" => Some(Format::Gcz),
        "rvz" | "RVZ" => Some(Format::Rvz),
        "wbfs" | "WBFS" => Some(Format::Wbfs),
        "wia" | "WIA" => Some(Format::Wia),
        "tgc" | "TGC" => Some(Format::Tgc),
        _ => None,
    }
}

fn perform_blocking(
    disc_path: PathBuf,
    out_path: PathBuf,
    format_opts: FormatOptions,
    game_title: String,
    tx: smol::channel::Sender<String>,
) -> impl FnOnce() -> Result<(), Error> {
    move || {
        let disc_reader = get_disc_reader(&disc_path)?;
        let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;
        let mut out_writer = File::create(out_path)?.pipe(BufWriter::new);

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let status = format!("⤓  Exporting {game_title}  {progress_percentage:02}%");
                    let _ = tx.try_send(status);

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
        Ok(())
    }
}

pub fn export_game(game: Game, out_path: PathBuf) -> impl Straw<Game, String, Error> {
    sipper(async move |mut sender| {
        let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;

        let format_opts = out_path
            .extension()
            .and_then(OsStr::to_str)
            .and_then(ext_to_format)
            .ok_or(Error::InvalidFilename)?
            .pipe(FormatOptions::new);

        let game_title = game.title.to_string();

        let (tx, rx) = smol::channel::bounded(1);

        let handle = perform_blocking(disc_path, out_path, format_opts, game_title, tx)
            .pipe(std::thread::spawn);

        while let Ok(msg) = rx.recv().await {
            sender.send(msg).await;
        }

        handle.join().expect("Failed to join thread")?;

        Ok(game)
    })
}
