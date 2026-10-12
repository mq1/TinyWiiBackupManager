// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        disc_reader::{SharedMultiFileReader, retrieve_multi_disc_files},
        game::Game,
    },
    long_operation::LongOperationState,
    util::{misc::OPTIMAL_THREADS, sha1_list},
};
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;
use std::{fmt, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashOutcome {
    Known,
    Unknown,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct HashResult {
    pub game: Game,
    // No disc number means that the game's images could not be enumerated.
    pub disc: Option<u16>,
    pub sha1: Option<String>,
    pub outcome: HashOutcome,
}

impl HashResult {
    fn from_digest(game: &Game, disc: u16, digest: [u8; 20]) -> Self {
        Self {
            game: game.clone(),
            disc: Some(disc),
            sha1: Some(digest.iter().map(|byte| format!("{byte:02x}")).collect()),
            outcome: if sha1_list::is_known(&digest) {
                HashOutcome::Known
            } else {
                HashOutcome::Unknown
            },
        }
    }

    fn from_error(game: &Game, disc: Option<u16>, error: anyhow::Error) -> Self {
        Self {
            game: game.clone(),
            disc,
            sha1: None,
            outcome: HashOutcome::Error(format!("{error:#}")),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HashSummary {
    pub games: usize,
    pub known: usize,
    pub unknown: usize,
    pub errors: usize,
}

impl HashSummary {
    fn record(&mut self, result: &HashResult) {
        match &result.outcome {
            HashOutcome::Known => self.known += 1,
            HashOutcome::Unknown => self.unknown += 1,
            HashOutcome::Error(_) => self.errors += 1,
        }
    }
}

impl fmt::Display for HashSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Checked {} games: {} known hashes, {} unknown hashes, {} errors",
            self.games, self.known, self.unknown, self.errors
        )
    }
}

#[derive(Debug, Clone)]
pub enum BatchHashEvent {
    Progress(String),
    Result(HashResult),
    Finished(HashSummary),
}

fn hash_disc(source: &Path, mut progress: impl FnMut(u64)) -> Result<[u8; 20]> {
    let files = retrieve_multi_disc_files(source).context("opening source disc")?;
    let stream = SharedMultiFileReader::new(files)?;
    let reader = DiscReader::new_stream(
        Box::new(stream),
        &DiscOptions {
            preloader_threads: OPTIMAL_THREADS.preloader,
            ..Default::default()
        },
    )?;
    // Hash the reconstructed ISO data, without writing an output image.
    let writer = DiscWriter::new(reader, &FormatOptions::default())?;
    let mut previous_percentage = None;
    let finalization = writer.process(
        |_, bytes, total| {
            let percentage = bytes.saturating_mul(100).checked_div(total).unwrap_or(0);
            if previous_percentage != Some(percentage) {
                progress(percentage);
                previous_percentage = Some(percentage);
            }
            Ok(())
        },
        &ProcessOptions {
            digest_sha1: true,
            ..Default::default()
        },
    )?;
    finalization.sha1.context("Failed to calculate SHA1")
}

fn perform_batch_blocking(games: &[Game], tx: &smol::channel::Sender<BatchHashEvent>) {
    let mut summary = HashSummary {
        games: games.len(),
        ..Default::default()
    };
    for (index, game) in games.iter().enumerate() {
        let label = format!(
            "{}/{}: {} [{}]",
            index + 1,
            games.len(),
            game.title(),
            game.id()
        );
        if tx
            .send_blocking(BatchHashEvent::Progress(format!("✓  Hashing {label}")))
            .is_err()
        {
            return;
        }
        let discs = match game.get_disc_paths_blocking() {
            Ok(discs) => discs,
            Err(error) => {
                let result = HashResult::from_error(game, None, error);
                summary.record(&result);
                if tx.send_blocking(BatchHashEvent::Result(result)).is_err() {
                    return;
                }
                continue;
            }
        };
        for (number, source) in discs {
            let label = format!("{label} (Disc {number})");
            let result = match hash_disc(&source, |percentage| {
                let _ = tx.try_send(BatchHashEvent::Progress(format!(
                    "✓  Hashing {label}  {percentage:02}%"
                )));
            }) {
                Ok(digest) => HashResult::from_digest(game, number, digest),
                Err(error) => HashResult::from_error(game, Some(number), error),
            };
            summary.record(&result);
            if tx.send_blocking(BatchHashEvent::Result(result)).is_err() {
                return;
            }
        }
    }
    // Per-disc results never complete the overall operation.
    let _ = tx.send_blocking(BatchHashEvent::Finished(summary));
}

pub fn calc_sha1_games(games: Vec<Game>) -> impl Stream<Item = BatchHashEvent> {
    let (tx, rx) = smol::channel::bounded(1);
    let _ = std::thread::spawn(move || perform_batch_blocking(&games, &tx));
    rx
}

fn perform_blocking(game: &Game, tx: &smol::channel::Sender<LongOperationState>) -> Result<bool> {
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
                let _ = tx.try_send(LongOperationState::Progress(format!(
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

pub fn calc_sha1(game: Game) -> impl Stream<Item = LongOperationState> {
    let (tx, rx) = smol::channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let exit = match perform_blocking(&game, &tx) {
            Ok(true) => LongOperationState::Finished(format!(
                "Hash match for {}!  -  SHA1 is well known, your dump is perfect",
                game.title()
            )),
            Ok(false) => LongOperationState::Errored(format!("Hash mismatch for {}", game.title())),
            Err(e) => LongOperationState::Errored(e.to_string()),
        };

        tx.send_blocking(exit).expect("Channel should be open");
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use nod::common::Format;
    use smol::stream::StreamExt;
    use std::io::{Cursor, Seek, Write};

    // Independently calculated SHA1 for this exact 64 KiB fixture.
    const SYNTHETIC_SHA1: &str = "43b7c6f446200a770f8961331197119527d6decb";

    fn synthetic_disc() -> Vec<u8> {
        let mut data = vec![0; 0x10000];
        data[..6].copy_from_slice(b"GZZE01");
        data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        data
    }

    fn game(base: &Path, title: &str, id: &str) -> Game {
        let path = base.join(format!("{title} [{id}]"));
        std::fs::create_dir(&path).unwrap();
        smol::block_on(Game::try_from_path(path, false)).unwrap()
    }

    fn collect(games: Vec<Game>) -> Vec<BatchHashEvent> {
        smol::block_on(calc_sha1_games(games).collect::<Vec<_>>())
    }

    fn results(events: &[BatchHashEvent]) -> Vec<&HashResult> {
        events
            .iter()
            .filter_map(|event| match event {
                BatchHashEvent::Result(result) => Some(result),
                _ => None,
            })
            .collect()
    }

    fn encode_disc(source: &Path, format: Format) -> Vec<u8> {
        let reader = DiscReader::new(source, &DiscOptions::default()).unwrap();
        let writer = DiscWriter::new(reader, &FormatOptions::new(format)).unwrap();
        let mut output = Cursor::new(Vec::new());
        let finalization = writer
            .process(
                |data, _, _| output.write_all(&data),
                &ProcessOptions::default(),
            )
            .unwrap();
        if !finalization.header.is_empty() {
            output.rewind().unwrap();
            output.write_all(&finalization.header).unwrap();
        }
        output.into_inner()
    }

    #[test]
    fn synthetic_hash_matches_independent_digest_and_preserves_source() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Synthetic", "GZZE01");
        let data = synthetic_disc();
        let source = game.path().join("game.iso");
        std::fs::write(&source, &data).unwrap();

        let events = collect(vec![game.clone()]);
        let results = results(&events);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].game.path(), game.path());
        assert_eq!(results[0].disc, Some(1));
        assert_eq!(results[0].sha1.as_deref(), Some(SYNTHETIC_SHA1));
        assert_eq!(results[0].outcome, HashOutcome::Unknown);
        assert_eq!(std::fs::read(source).unwrap(), data);
        assert_eq!(std::fs::read_dir(game.path()).unwrap().count(), 1);
        assert!(matches!(
            events.last(),
            Some(BatchHashEvent::Finished(HashSummary {
                games: 1,
                known: 0,
                unknown: 1,
                errors: 0,
            }))
        ));
    }

    #[test]
    fn known_digest_uses_bundled_database() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Known", "GZZE01");
        let known_hashes = include_bytes!(concat!(env!("OUT_DIR"), "/sha1_list.bin"));
        let digest = known_hashes[..20].try_into().unwrap();
        let result = HashResult::from_digest(&game, 2, digest);
        assert_eq!(result.outcome, HashOutcome::Known);
        assert_eq!(result.disc, Some(2));
        assert_eq!(result.sha1.as_ref().unwrap().len(), 40);
        let mut summary = HashSummary::default();
        summary.record(&result);
        assert_eq!(summary.known, 1);
    }

    #[test]
    fn batch_continues_after_missing_game_and_bad_primary_to_secondary_discs() {
        let temp = tempfile::tempdir().unwrap();
        let missing = game(temp.path(), "Missing", "GZZP01");
        let damaged = game(temp.path(), "Damaged primary", "GZZJ01");
        let valid = game(temp.path(), "Valid", "GZZE01");
        let data = synthetic_disc();
        let bad_data = vec![0; data.len()];
        std::fs::write(damaged.path().join("game.iso"), &bad_data).unwrap();
        std::fs::write(damaged.path().join("disc2.iso"), &data).unwrap();
        std::fs::write(valid.path().join("game.iso"), &data).unwrap();

        let events = collect(vec![missing.clone(), damaged.clone(), valid.clone()]);
        let results = results(&events);
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].game.path(), missing.path());
        assert_eq!(results[0].disc, None);
        assert!(matches!(&results[0].outcome, HashOutcome::Error(_)));
        assert_eq!(results[1].game.path(), damaged.path());
        assert_eq!(results[1].disc, Some(1));
        assert!(matches!(&results[1].outcome, HashOutcome::Error(_)));
        assert_eq!(results[1].sha1, None);
        assert_eq!(results[2].disc, Some(2));
        assert_eq!(results[2].sha1.as_deref(), Some(SYNTHETIC_SHA1));
        assert_eq!(results[3].game.path(), valid.path());
        assert_eq!(results[3].sha1.as_deref(), Some(SYNTHETIC_SHA1));
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, BatchHashEvent::Finished(_)))
                .count(),
            1
        );
        assert!(matches!(
            events.last(),
            Some(BatchHashEvent::Finished(HashSummary {
                games: 3,
                known: 0,
                unknown: 2,
                errors: 2,
            }))
        ));
        assert_eq!(std::fs::read_dir(missing.path()).unwrap().count(), 0);
        assert_eq!(std::fs::read_dir(damaged.path()).unwrap().count(), 2);
        assert_eq!(std::fs::read_dir(valid.path()).unwrap().count(), 1);
        assert_eq!(
            std::fs::read(damaged.path().join("game.iso")).unwrap(),
            bad_data
        );
        assert_eq!(
            std::fs::read(damaged.path().join("disc2.iso")).unwrap(),
            data
        );
        assert_eq!(std::fs::read(valid.path().join("game.iso")).unwrap(), data);
    }

    #[test]
    fn split_iso_is_joined_and_missing_part_is_reported() {
        let temp = tempfile::tempdir().unwrap();
        let missing = game(temp.path(), "Missing part", "GZZP01");
        let valid = game(temp.path(), "Split ISO", "GZZE01");
        let data = synthetic_disc();
        std::fs::write(missing.path().join("GZZP01.part0.iso"), &data[..0x8000]).unwrap();
        std::fs::write(valid.path().join("GZZE01.part0.iso"), &data[..0x8000]).unwrap();
        std::fs::write(valid.path().join("GZZE01.part1.iso"), &data[0x8000..]).unwrap();

        let events = collect(vec![missing, valid.clone()]);
        let results = results(&events);
        assert_eq!(results.len(), 2);
        assert!(matches!(&results[0].outcome, HashOutcome::Error(_)));
        assert_eq!(results[1].sha1.as_deref(), Some(SYNTHETIC_SHA1));
        assert_eq!(std::fs::read_dir(valid.path()).unwrap().count(), 2);
        assert_eq!(
            std::fs::read(valid.path().join("GZZE01.part0.iso")).unwrap(),
            data[..0x8000]
        );
        assert_eq!(
            std::fs::read(valid.path().join("GZZE01.part1.iso")).unwrap(),
            data[0x8000..]
        );
    }

    #[test]
    fn split_wbfs_hashes_reconstructed_disc_and_preserves_parts() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Split WBFS", "GZZE01");
        let source = temp.path().join("fixture.iso");
        std::fs::write(&source, synthetic_disc()).unwrap();
        let wbfs = encode_disc(&source, Format::Wbfs);
        let split = wbfs.len() / 2;
        std::fs::write(game.path().join("GZZE01.wbfs"), &wbfs[..split]).unwrap();
        std::fs::write(game.path().join("GZZE01.wbf1"), &wbfs[split..]).unwrap();

        let events = collect(vec![game.clone()]);
        let results = results(&events);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].sha1.as_deref(), Some(SYNTHETIC_SHA1));
        assert_eq!(std::fs::read_dir(game.path()).unwrap().count(), 2);
        assert_eq!(
            std::fs::read(game.path().join("GZZE01.wbfs")).unwrap(),
            wbfs[..split]
        );
        assert_eq!(
            std::fs::read(game.path().join("GZZE01.wbf1")).unwrap(),
            wbfs[split..]
        );
    }

    #[test]
    fn compressed_disc_hashes_iso_data() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("fixture.iso");
        std::fs::write(&source, synthetic_disc()).unwrap();
        let rvz = encode_disc(&source, Format::Rvz);
        let compressed = temp.path().join("fixture.rvz");
        std::fs::write(&compressed, &rvz).unwrap();
        let digest = hash_disc(&compressed, |_| {}).unwrap();
        let sha1: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(sha1, SYNTHETIC_SHA1);
        assert_eq!(std::fs::read(compressed).unwrap(), rvz);
        assert_eq!(std::fs::read(source).unwrap(), synthetic_disc());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn empty_batch_has_one_terminal_event() {
        let events = collect(Vec::new());
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events.last(),
            Some(BatchHashEvent::Finished(summary)) if summary == &HashSummary::default()
        ));
    }
}
