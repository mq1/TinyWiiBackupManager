// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

pub mod banners;
pub mod calc_sha1;
pub mod convert;
pub mod covers;
pub mod disc_reader;
pub mod export;
pub mod game;
pub mod games_state;
pub mod import;
pub mod scrub;
pub mod txtcodes;

#[derive(Debug, Clone)]
pub struct ImportEntry {
    path: PathBuf,
    meta: wii_disc_info::Meta,
    fingerprint: [u8; 5],
}

impl ImportEntry {
    pub async fn new(path: PathBuf) -> Result<Self> {
        let is_zip = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

        let meta = if is_zip {
            smol::unblock({
                let path = path.clone();

                move || {
                    let f = std::fs::File::open(path)?;
                    let mut archive = ZipArchive::new(f)?;
                    let mut first_entry = archive.by_index(0)?;
                    let meta = wii_disc_info::Meta::read(&mut first_entry)?;
                    Ok::<_, anyhow::Error>(meta)
                }
            })
            .await?
        } else {
            let mut f = File::open(&path).await?;
            wii_disc_info::Meta::read_async(&mut f).await?
        };

        let mut fingerprint = [0u8; 5];

        let id_compact = meta.game_id().to_u32().to_be_bytes();
        fingerprint[0..4].copy_from_slice(&id_compact);

        let is_disc1 = is_x(&path, "(Disc 1)");
        let is_disc2 = is_x(&path, "(Disc 2)");

        let flags = (is_disc1 as u8) << 1 | (is_disc2 as u8);
        fingerprint[4] = flags;

        Ok(Self {
            path,
            meta,
            fingerprint,
        })
    }

    #[must_use]
    #[inline]
    fn is_multidisc(&self) -> bool {
        self.fingerprint[4] != 0
    }

    #[must_use]
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    #[inline]
    pub fn meta(&self) -> &wii_disc_info::Meta {
        &self.meta
    }

    /// The game's title, with the disc when the game has more than one, so the discs of one game
    /// can be told apart in the task manager.
    #[must_use]
    pub fn display_title(&self) -> String {
        let title = self.meta.game_title();
        if self.is_multidisc() || self.meta.disc_number() > 0 {
            format!("{title} (Disc {})", self.meta.disc_number() + 1)
        } else {
            title.to_string()
        }
    }
}

impl PartialEq for ImportEntry {
    fn eq(&self, other: &Self) -> bool {
        self.fingerprint == other.fingerprint
    }
}

impl Eq for ImportEntry {}

impl PartialOrd for ImportEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ImportEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fingerprint.cmp(&other.fingerprint)
    }
}

fn is_x(path: &Path, x: &str) -> bool {
    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|s| s.contains(x))
}

impl ImportEntry {
    /// The disc this entry writes on the drive: two entries with the same one (e.g. the same game
    /// picked twice, or from two files) would write the same file.
    pub fn target(&self) -> (GameID, u8) {
        (self.meta.game_id(), self.meta.disc_number())
    }

    /// Whether this disc is already in its game's folder on the drive (a file named like the one
    /// the import would write). Importing it again would overwrite the game, and a cancelled or
    /// failed import would then leave it broken.
    pub fn is_on_drive(&self, mount_point: &Path) -> bool {
        if mount_point.as_os_str().is_empty() {
            return false;
        }

        let (parent, prefix) = if self.meta.is_wii() {
            ("wbfs", format!("{}.", self.meta.game_id()))
        } else {
            match self.meta.disc_number() {
                0 => ("games", "game.".to_string()),
                n => ("games", format!("disc{}.", n + 1)),
            }
        };

        let dir = mount_point.join(parent).join(import::make_game_dir_name(
            self.meta.game_id(),
            self.meta.game_title(),
        ));

        std::fs::read_dir(dir).is_ok_and(|entries| {
            entries
                .flatten()
                .any(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        })
    }
}

pub async fn keep_valid_games(
    games: impl Stream<Item = PathBuf>,
    existing_ids: &[GameID],
) -> Vec<ImportEntry> {
    let mut entries = games
        .then(ImportEntry::new)
        .filter_map(Result::ok)
        .collect::<Vec<_>>()
        .await;

    // remove duplicates
    entries.sort_unstable();
    entries.dedup();

    // filter out existing games
    entries.retain(|entry| {
        let exists = existing_ids.contains(&entry.meta.game_id());

        // if it's a dual disc, try anyways
        !exists || entry.is_multidisc()
    });

    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A GameCube disc image with only its header filled in.
    fn gamecube_disc(path: &Path, game_id: &[u8; 6], disc_number: u8) {
        let mut image = vec![0u8; 0x10000];
        image[..6].copy_from_slice(game_id);
        image[6] = disc_number;
        image[0x1c..0x20].copy_from_slice(&[0xC2, 0x33, 0x9F, 0x3D]);
        image[0x20..0x29].copy_from_slice(b"Test Game");
        std::fs::write(path, image).unwrap();
    }

    /// A Wii disc image with only its header filled in.
    fn wii_disc(path: &Path, game_id: &[u8; 6]) {
        let mut image = vec![0u8; 0x10000];
        image[..6].copy_from_slice(game_id);
        image[0x18..0x1c].copy_from_slice(&[0x5D, 0x1C, 0x9E, 0xA3]);
        image[0x20..0x29].copy_from_slice(b"Test Game");
        std::fs::write(path, image).unwrap();
    }

    #[test]
    fn a_disc_on_the_drive_is_found_by_its_file() {
        let dir = tempfile::tempdir().unwrap();
        let drive = dir.path().join("drive");
        let disc1 = dir.path().join("disc1.iso");
        let disc2 = dir.path().join("disc2.iso");
        gamecube_disc(&disc1, b"GTSE01", 0);
        gamecube_disc(&disc2, b"GTSE01", 1);
        let disc1 = smol::block_on(ImportEntry::new(disc1)).unwrap();
        let disc2 = smol::block_on(ImportEntry::new(disc2)).unwrap();

        assert!(!disc1.is_on_drive(&drive));

        let game_dir = drive.join("games").join(import::make_game_dir_name(
            disc1.meta().game_id(),
            disc1.meta().game_title(),
        ));
        std::fs::create_dir_all(&game_dir).unwrap();
        std::fs::write(game_dir.join("game.iso"), b"").unwrap();

        // the first disc is there, the second may still be added
        assert!(disc1.is_on_drive(&drive));
        assert!(!disc2.is_on_drive(&drive));
        // no drive selected
        assert!(!disc1.is_on_drive(Path::new("")));
    }

    #[test]
    fn a_split_wii_game_on_the_drive_is_found() {
        let dir = tempfile::tempdir().unwrap();
        let drive = dir.path().join("drive");
        let image = dir.path().join("game.iso");
        wii_disc(&image, b"RTSP01");
        let entry = smol::block_on(ImportEntry::new(image)).unwrap();

        let game_dir = drive.join("wbfs").join(import::make_game_dir_name(
            entry.meta().game_id(),
            entry.meta().game_title(),
        ));
        std::fs::create_dir_all(&game_dir).unwrap();
        std::fs::write(game_dir.join("notes.txt"), b"").unwrap();
        assert!(!entry.is_on_drive(&drive));

        std::fs::write(game_dir.join("RTSP01.part0.iso"), b"").unwrap();
        assert!(entry.is_on_drive(&drive));
    }
}
