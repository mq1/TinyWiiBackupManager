// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{conversion_state::ConversionState, game::Game},
    util::sha1_list,
};
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;
use smol_str::{ToSmolStr, format_smolstr};

fn perform_blocking(game: Game, tx: smol::channel::Sender<ConversionState>) -> Result<bool> {
    let disc_path = game.get_disc_path_blocking().context("disc not found")?;

    let disc = DiscReader::new(&disc_path, &DiscOptions::default())?;

    let process_opts = ProcessOptions {
        digest_sha1: true,
        ..Default::default()
    };

    let writer = DiscWriter::new(disc, &FormatOptions::default())?;

    let mut prev_percentage = 100;
    let finalization = writer.process(
        |_, progress, total| {
            let progress_percentage = progress * 100 / total;

            if progress_percentage != prev_percentage {
                let _ = tx.try_send(ConversionState::Progress(format_smolstr!(
                    "✓  Hashing {}  {progress_percentage:02}%",
                    game.title()
                )));

                prev_percentage = progress_percentage;
            }

            Ok(())
        },
        &process_opts,
    )?;

    let sha1 = finalization.sha1.context("Failed to calculate SHA1")?;

    let known_sha1 = sha1_list::is_known(&sha1);

    Ok::<_, anyhow::Error>(known_sha1)
}

pub fn calc_sha1(game: Game) -> impl Stream<Item = ConversionState> {
    let (tx, rx) = smol::channel::bounded(1);
    let game_title = game.title_cloned();

    let _ = std::thread::spawn(move || match perform_blocking(game, tx) {
        Ok(true) => ConversionState::Finished(format_smolstr!(
            "Hash match for {game_title}!  -  SHA1 is well known, your dump is perfect",
        )),
        Ok(false) => ConversionState::Errored(format_smolstr!("Hash mismatch for {game_title}")),
        Err(e) => ConversionState::Errored(e.to_smolstr()),
    });

    rx
}
