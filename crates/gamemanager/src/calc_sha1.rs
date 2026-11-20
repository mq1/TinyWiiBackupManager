// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};

pub fn calc_sha1(game: &Game, send_progress: impl Fn(String)) -> Result<String> {
    let disc_path = game.get_disc_path().context("disc not found")?;

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
                send_progress(format!(
                    "✓  Hashing {}  {progress_percentage:02}%",
                    game.title()
                ));

                prev_percentage = progress_percentage;
            }

            Ok(())
        },
        &process_opts,
    )?;

    let sha1 = finalization.sha1.context("Failed to calculate SHA1")?;

    let msg = if twbm_sha1list::is_known(&sha1) {
        format!(
            "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
            game.title()
        )
    } else {
        format!("Hash mismatch for {}", game.title())
    };

    Ok(msg)
}
