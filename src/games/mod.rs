// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use smol::{
    fs::File,
    stream::{Stream, StreamExt},
};
use std::{
    ffi::OsStr,
    io::{self, Read, Seek},
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
pub mod transfer;
pub mod txtcodes;

// Metadata discovery and extraction must select the same member. ZIP archives
// commonly place a directory or README before their game image.
pub(crate) fn zip_game_index<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    mut check: impl FnMut() -> io::Result<()>,
) -> Result<usize> {
    let mut first_file = None;
    let mut image = None;
    for index in 0..archive.len() {
        check()?;
        let entry = archive.by_index_data(index)?;
        if !entry.is_file() {
            continue;
        }
        first_file.get_or_insert(index);
        let name = entry.name()?;
        let extension = Path::new(name.as_ref())
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        if ["wbf1", "wbf2", "wbf3"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported))
        {
            bail!("Extract split game images from the ZIP before importing them");
        }
        if ["iso", "gcm", "wia", "rvz", "wbfs", "ciso", "gcz", "tgc"]
            .iter()
            .any(|supported| extension.eq_ignore_ascii_case(supported))
            && image.replace(index).is_some()
        {
            bail!("ZIP contains multiple game images; extract them before importing");
        }
    }
    check()?;
    // Preserve support for a single image whose member has no usual extension;
    // ImportEntry validates the selected member's actual disc header below.
    image.or(first_file).context("ZIP contains no game image")
}

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
                    let index = zip_game_index(&mut archive, || Ok(()))?;
                    let mut image = archive.by_index(index)?;
                    let meta = wii_disc_info::Meta::read(&mut image)?;
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

        // Identify the actual destination disc, independently of its filename.
        // Wii images share one destination per ID; GameCube discs use game/disc2.
        fingerprint[4] = if meta.is_wii() { 0 } else { meta.disc_number() };

        Ok(Self {
            path,
            meta,
            fingerprint,
        })
    }

    #[must_use]
    #[inline]
    fn is_multidisc(&self) -> bool {
        self.meta.disc_number() != 0 || is_x(&self.path, "(Disc 1)") || is_x(&self.path, "(Disc 2)")
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
    use crate::{
        config::Config,
        games::{
            games_state::GamesState,
            import::{BatchImportEvent, ImportCancellation, import_games},
        },
        util::drive_state::DriveState,
    };
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};

    fn entry(dir: &Path, name: &str, disc: u8) -> ImportEntry {
        let path = dir.join(name);
        let mut data = vec![0; 0x8000];
        data[..6].copy_from_slice(b"GZZE01");
        data[6] = disc;
        data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        data[0x20..0x24].copy_from_slice(b"Test");
        std::fs::write(&path, data).unwrap();
        smol::block_on(ImportEntry::new(path)).unwrap()
    }

    #[test]
    fn import_identity_uses_disc_metadata_instead_of_filename() {
        let dir = tempfile::tempdir().unwrap();
        let disc1 = entry(dir.path(), "first.iso", 0);
        let disc2 = entry(dir.path(), "second.iso", 1);
        let renamed_disc1 = entry(dir.path(), "misleading (Disc 2).iso", 0);
        assert_ne!(disc1, disc2);
        assert_eq!(disc1, renamed_disc1);
    }

    #[test]
    fn picker_keeps_both_discs_without_disc_labels_and_deduplicates_copies() {
        let dir = tempfile::tempdir().unwrap();
        let first = entry(dir.path(), "first.iso", 0);
        let second = entry(dir.path(), "second.iso", 1);
        let duplicate = entry(dir.path(), "copy (Disc 2).iso", 0);
        let paths = [first.path, second.path, duplicate.path];
        let entries = smol::block_on(keep_valid_games(smol::stream::iter(paths), &[]));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].meta.disc_number(), 0);
        assert_eq!(entries[1].meta.disc_number(), 1);
    }

    fn import_zip_with_leading_entry(directory_first: bool) {
        let temp = tempfile::tempdir().unwrap();
        let original = entry(temp.path(), "original.iso", 0);
        let image = std::fs::read(original.path()).unwrap();
        let source = temp.path().join("source.zip");
        let mut zip = ZipWriter::new(std::fs::File::create(&source).unwrap());
        let options = SimpleFileOptions::default();
        if directory_first {
            zip.add_directory("images/", options).unwrap();
        } else {
            zip.start_file("README.txt", options).unwrap();
            zip.write_all(b"This archive contains one game.").unwrap();
        }
        zip.start_file("images/actual.GCM", options).unwrap();
        zip.write_all(&image).unwrap();
        zip.finish().unwrap();
        let source_before = std::fs::read(&source).unwrap();
        let imported = smol::block_on(ImportEntry::new(source.clone())).unwrap();
        assert_eq!(imported.meta().game_id(), original.meta().game_id());
        let mount = temp.path().join("drive");
        std::fs::create_dir(&mount).unwrap();
        let mut config = Config::default();
        drop(config.set_mount_point(mount.clone()));
        let events = smol::block_on(
            import_games(
                vec![imported],
                config,
                DriveState::NotLoaded,
                1,
                ImportCancellation::default(),
            )
            .collect::<Vec<_>>(),
        );
        assert!(
            matches!(events.last(), Some(BatchImportEvent::Finished(summary)) if summary.imported == 1 && summary.failed == 0 && summary.cancelled == 0),
            "{events:?}"
        );
        let GamesState::Loaded(games) = smol::block_on(GamesState::load(mount)) else {
            panic!("Game scan failed")
        };
        assert_eq!(games.count(), 1);
        let output = games
            .iter_all()
            .next()
            .unwrap()
            .get_disc_path_blocking()
            .unwrap();
        assert_eq!(std::fs::read(output).unwrap(), image);
        assert_eq!(std::fs::read(source).unwrap(), source_before);
    }

    #[test]
    fn game_zip_with_leading_directory_imports_the_image_completely() {
        import_zip_with_leading_entry(true);
    }

    #[test]
    fn game_zip_with_leading_readme_imports_the_image_completely() {
        import_zip_with_leading_entry(false);
    }

    #[test]
    fn game_zip_with_multiple_images_is_rejected_without_changes() {
        let temp = tempfile::tempdir().unwrap();
        let original = entry(temp.path(), "original.iso", 0);
        let image = std::fs::read(original.path()).unwrap();
        let source = temp.path().join("multiple.zip");
        let mut zip = ZipWriter::new(std::fs::File::create(&source).unwrap());
        for filename in ["one.iso", "two.iso"] {
            zip.start_file(filename, SimpleFileOptions::default())
                .unwrap();
            zip.write_all(&image).unwrap();
        }
        zip.finish().unwrap();
        let before = std::fs::read(&source).unwrap();
        assert!(smol::block_on(ImportEntry::new(source.clone())).is_err());
        assert_eq!(std::fs::read(source).unwrap(), before);
    }
}
