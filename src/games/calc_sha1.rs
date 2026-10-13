// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, long_operation::LongOperationState, util::sha1_list};
use anyhow::{Context, Result};
use nod::{
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use smol::stream::Stream;
use std::path::Path;

fn hash_disc(source: &Path, mut progress: impl FnMut(u64)) -> Result<[u8; 20]> {
    // Require a continuation for .part0.iso, while preserving nod's priority
    // for dotted .iso.1 continuations and its discovery of later parts.
    if let Some(base) = source
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".part0.iso"))
    {
        let dotted_part = source.with_file_name(format!("{base}.part0.iso.1"));
        let has_dotted_part = match std::fs::metadata(&dotted_part) {
            Ok(metadata) => metadata.is_file(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error).context("checking source continuation"),
        };
        if !has_dotted_part {
            drop(
                std::fs::File::open(source.with_file_name(format!("{base}.part1.iso")))
                    .context("opening source disc")?,
            );
        }
    }
    let reader = DiscReader::new(source, &DiscOptions::default()).context("opening source disc")?;
    // Hash the reconstructed ISO data without writing an output image.
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

fn perform_blocking(game: &Game, tx: &smol::channel::Sender<LongOperationState>) -> Result<bool> {
    let mut all_known = true;
    for (number, source) in game.get_disc_paths_blocking()? {
        let digest = hash_disc(&source, |percentage| {
            let _ = tx.try_send(LongOperationState::Progress(format!(
                "✓  Hashing {} (Disc {number})  {percentage:02}%",
                game.title()
            )));
        })
        .with_context(|| format!("Could not check disc {number}"))?;
        all_known &= sha1_list::is_known(&digest);
    }
    Ok(all_known)
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
            Err(e) => LongOperationState::Errored(format!("{e:#}")),
        };
        let _ = tx.send_blocking(exit);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::stream::StreamExt;

    const SYNTHETIC_SHA1: &str = "43b7c6f446200a770f8961331197119527d6decb";

    fn synthetic_disc() -> Vec<u8> {
        let mut data = vec![0; 0x10000];
        data[..6].copy_from_slice(b"GZZE01");
        data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        data
    }

    fn game(base: &Path) -> Game {
        let path = base.join("Synthetic [GZZE01]");
        std::fs::create_dir(&path).unwrap();
        smol::block_on(Game::try_from_path(path, false)).unwrap()
    }

    fn assert_split_hash(names: &[&str], chunk_size: usize) {
        let dir = tempfile::tempdir().unwrap();
        let data = synthetic_disc();
        let mut offset = 0;
        let mut parts = Vec::new();
        for (index, name) in names.iter().enumerate() {
            let end = if index + 1 == names.len() {
                data.len()
            } else {
                offset + chunk_size
            };
            let bytes = &data[offset..end];
            let path = dir.path().join(name);
            std::fs::write(&path, bytes).unwrap();
            parts.push((path, bytes));
            offset = end;
        }
        let digest = hash_disc(&parts[0].0, |_| {}).unwrap();
        let sha1: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        for (path, bytes) in parts {
            assert_eq!(std::fs::read(path).unwrap(), bytes);
        }
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), names.len());
        assert_eq!(sha1, SYNTHETIC_SHA1);
    }

    #[test]
    fn hash_supports_upstream_iso_numbered_parts() {
        assert_split_hash(&["game.iso", "game.iso.1", "game.iso.2"], 0x4000);
    }

    #[test]
    fn hash_supports_upstream_numbered_part_zero_continuations() {
        assert_split_hash(&["game.part0.iso", "game.part0.iso.1"], 0x8000);
    }
    #[test]
    fn hash_supports_upstream_more_than_four_iso_parts() {
        assert_split_hash(
            &[
                "game.part0.iso",
                "game.part1.iso",
                "game.part2.iso",
                "game.part3.iso",
                "game.part4.iso",
                "game.part5.iso",
            ],
            0x2000,
        );
    }
    #[test]
    fn hash_joins_three_iso_parts_and_preserves_source() {
        let dir = tempfile::tempdir().unwrap();
        let data = synthetic_disc();
        let parts = [&data[..0x4000], &data[0x4000..0x8000], &data[0x8000..]];
        for (index, bytes) in parts.iter().enumerate() {
            std::fs::write(dir.path().join(format!("game.part{index}.iso")), bytes).unwrap();
        }
        let digest = hash_disc(&dir.path().join("game.part0.iso"), |_| {}).unwrap();
        let sha1: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(sha1, SYNTHETIC_SHA1);
        for (index, bytes) in parts.iter().enumerate() {
            assert_eq!(
                std::fs::read(dir.path().join(format!("game.part{index}.iso"))).unwrap(),
                *bytes
            );
        }
    }

    #[test]
    fn missing_iso_part_is_a_read_error_instead_of_a_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let game = game(dir.path());
        let primary = game.path().join("GZZE01.part0.iso");
        let data = synthetic_disc();
        std::fs::write(&primary, &data[..0x8000]).unwrap();
        let events = smol::block_on(calc_sha1(game).collect::<Vec<_>>());
        assert!(
            matches!(events.last(), Some(LongOperationState::Errored(message))
            if message.contains("Could not check disc 1") && message.contains("opening source disc"))
        );
        assert_eq!(std::fs::read(primary).unwrap(), data[..0x8000]);
    }

    #[test]
    fn second_gamecube_disc_is_checked_even_if_first_hash_is_unknown() {
        let dir = tempfile::tempdir().unwrap();
        let game = game(dir.path());
        let data = synthetic_disc();
        std::fs::write(game.path().join("game.iso"), &data).unwrap();
        std::fs::write(game.path().join("disc2.iso"), [0; 32]).unwrap();
        let events = smol::block_on(calc_sha1(game.clone()).collect::<Vec<_>>());
        assert!(
            matches!(events.last(), Some(LongOperationState::Errored(message))
            if message.contains("Could not check disc 2"))
        );
        assert_eq!(std::fs::read(game.path().join("game.iso")).unwrap(), data);
        assert_eq!(
            std::fs::read(game.path().join("disc2.iso")).unwrap(),
            [0; 32]
        );
    }
}
