// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    tasks::{self, Reporter, TaskControl, WorkerEvent},
    util::sha1_list,
};
use anyhow::{Context, Result};
use nod::{
    common::PartitionKind,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;
use std::path::Path;

/// What the hash says about a dump.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// The SHA-1 is in the list of known good dumps (Redump).
    Known,
    /// A Wii dump without its update partition (WBFS tools often drop it): it can't match the
    /// disc any more, whether or not the rest is intact.
    UpdateRemoved,
    /// Not in the list: damaged, another version, or changed in a way the hash can't tell.
    Unknown,
}

/// Whether a Wii disc's partitions lack the update partition; GameCube discs have none at all.
fn lacks_update_partition(kinds: &[PartitionKind]) -> bool {
    !kinds.is_empty() && !kinds.contains(&PartitionKind::Update)
}

/// Hashes one disc image.
fn hash_disc(
    disc_path: &Path,
    control: &TaskControl,
    reporter: &mut Reporter,
    message: impl Fn() -> String,
) -> Result<Verdict> {
    reporter.progress(0, None, &message);

    let disc = DiscReader::new(disc_path, &DiscOptions::default())?;
    let kinds = disc.partitions().iter().map(|p| p.kind).collect::<Vec<_>>();

    let process_opts = ProcessOptions {
        digest_sha1: true,
        ..Default::default()
    };

    let writer = DiscWriter::new(disc, &FormatOptions::default())?;

    let finalization = writer.process(
        |_, progress, total| {
            control.checkpoint().map_err(std::io::Error::other)?;
            reporter.progress(progress, Some(total), &message);
            Ok(())
        },
        &process_opts,
    );

    // nod wraps the error; the task manager knows it was cancelled anyway
    control.checkpoint()?;
    let sha1 = finalization?.sha1.context("Failed to calculate SHA1")?;

    Ok(if sha1_list::is_known(&sha1) {
        Verdict::Known
    } else if lacks_update_partition(&kinds) {
        Verdict::UpdateRemoved
    } else {
        Verdict::Unknown
    })
}

/// Hashes every disc of the game; one verdict per disc, the first disc first.
fn perform_blocking(
    game: &Game,
    control: &TaskControl,
    reporter: &mut Reporter,
) -> Result<Vec<Verdict>> {
    let discs = game.get_disc_paths_blocking();
    if discs.is_empty() {
        anyhow::bail!("disc not found");
    }

    let count = discs.len();
    discs
        .iter()
        .enumerate()
        .map(|(i, disc)| {
            let message = || match count {
                1 => format!("Hashing {}", game.title()),
                _ => format!("Hashing {} (disc {} of {count})", game.title(), i + 1),
            };
            hash_disc(disc, control, reporter, message)
        })
        .collect()
}

/// What the task says: a match only when every disc matches, otherwise why the first
/// mismatching disc doesn't (named when there are several).
fn outcome(title: &str, verdicts: &[Verdict]) -> Result<String, String> {
    let Some((index, verdict)) = verdicts
        .iter()
        .enumerate()
        .find(|(_, verdict)| **verdict != Verdict::Known)
    else {
        let discs = match verdicts.len() {
            1 => String::new(),
            n => format!(" (all {n} discs)"),
        };
        return Ok(format!(
            "Hash match for {title}{discs}!  -  SHA1 is well known, your dump is perfect"
        ));
    };

    // the task's title already names the game
    let which = match verdicts.len() {
        1 => String::new(),
        _ => format!(" on disc {}", index + 1),
    };
    Err(match verdict {
        Verdict::UpdateRemoved => format!(
            "Hash mismatch{which}: the dump has no update partition (often removed when making a \
             WBFS), so it can't match the disc. That alone doesn't stop the game from working; \
             whether the rest is intact, the hash can't tell"
        ),
        _ => format!(
            "Hash mismatch{which}: not a known good dump. It may be damaged, another version or \
             changed otherwise; the hash can't tell which"
        ),
    })
}

/// Checks the SHA-1 of each of the game's discs against the list of known good dumps as a task.
pub fn calc_sha1(game: Game, control: TaskControl) -> impl Stream<Item = WorkerEvent> {
    tasks::spawn_blocking(
        move |reporter| match perform_blocking(&game, &control, reporter) {
            Ok(verdicts) => outcome(game.title(), &verdicts),
            Err(e) => Err(e.to_string()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_wii_disc_without_update_partition_counts_as_shortened() {
        assert!(lacks_update_partition(&[PartitionKind::Data]));
        assert!(!lacks_update_partition(&[
            PartitionKind::Update,
            PartitionKind::Data
        ]));
        // GameCube: no partitions at all
        assert!(!lacks_update_partition(&[]));
    }

    #[test]
    fn every_disc_has_to_match() {
        assert!(outcome("Tales", &[Verdict::Known]).is_ok_and(|m| !m.contains("discs")));
        assert!(
            outcome("Tales", &[Verdict::Known, Verdict::Known])
                .is_ok_and(|m| m.contains("all 2 discs"))
        );

        let second_bad = outcome("Tales", &[Verdict::Known, Verdict::Unknown]).unwrap_err();
        assert!(
            second_bad.starts_with("Hash mismatch on disc 2:"),
            "{second_bad}"
        );

        let single_bad = outcome("Tales", &[Verdict::UpdateRemoved]).unwrap_err();
        assert!(single_bad.starts_with("Hash mismatch: the dump has no update partition"));
    }
}
