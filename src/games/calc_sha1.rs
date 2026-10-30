// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    tasks::{self, Reporter, TaskControl, WorkerEvent},
    util::sha1_list,
};
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;

fn perform_blocking(game: &Game, control: &TaskControl, reporter: &mut Reporter) -> Result<bool> {
    let disc_path = game.get_disc_path_blocking().context("disc not found")?;
    let message = || format!("Hashing {}", game.title());
    reporter.progress(0, None, message);

    let disc = DiscReader::new(&disc_path, &DiscOptions::default())?;

    let process_opts = ProcessOptions {
        digest_sha1: true,
        ..Default::default()
    };

    let writer = DiscWriter::new(disc, &FormatOptions::default())?;

    let finalization = writer.process(
        |_, progress, total| {
            control.checkpoint().map_err(std::io::Error::other)?;
            reporter.progress(progress, Some(total), message);
            Ok(())
        },
        &process_opts,
    );

    // nod wraps the error; the task manager knows it was cancelled anyway
    control.checkpoint()?;
    let sha1 = finalization?.sha1.context("Failed to calculate SHA1")?;

    Ok(sha1_list::is_known(&sha1))
}

/// Checks the game's SHA-1 against the list of known good dumps as a task.
pub fn calc_sha1(game: Game, control: TaskControl) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(
        move |reporter| match perform_blocking(&game, &control, reporter) {
            Ok(true) => Ok(format!(
                "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
                game.title()
            )),
            Ok(false) => Err(format!("Hash mismatch for {}", game.title())),
            Err(e) => Err(e.to_string()),
        },
    )
}
