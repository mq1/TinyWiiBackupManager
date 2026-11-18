// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game::Game;
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;
use twbm_common::Status;

fn perform_blocking(game: &Game, tx: &smol::channel::Sender<Status>) -> Result<bool> {
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
                let _ = tx.try_send(Status::Progress(
                    format!("✓  Hashing {}  {progress_percentage:02}%", game.title()).into(),
                ));

                prev_percentage = progress_percentage;
            }

            Ok(())
        },
        &process_opts,
    )?;

    let sha1 = finalization.sha1.context("Failed to calculate SHA1")?;

    let known_sha1 = twbm_sha1list::is_known(&sha1);

    Ok::<_, anyhow::Error>(known_sha1)
}

pub fn calc_sha1(game: Game) -> impl Stream<Item = Status> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = perform_blocking(&game, &tx)
            .map(|known_sha1| {
                if known_sha1 {
                    format!(
                        "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
                        game.title()
                    )
                } else {
                    format!("Hash mismatch for {}", game.title())
                }
            })
            .into();

        tx.send_blocking(exit).expect("Channel should be open");
    });

    rx
}
