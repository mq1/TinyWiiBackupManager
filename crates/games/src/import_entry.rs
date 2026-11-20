// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result};
use itertools::Itertools;
use std::{
    ffi::OsStr,
    fs::File,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct ImportEntry {
    pub path: PathBuf,
    pub meta: wii_disc_info::Meta,
    pub is_disc1: bool,
    pub is_disc2: bool,
}

impl ImportEntry {
    pub fn new(path: PathBuf) -> Result<Self> {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("Invalid filename")?;

        let meta = if filename.ends_with(".zip") || filename.ends_with(".ZIP") {
            let f = File::open(&path)?;
            let mut archive = ZipArchive::new(f)?;
            let mut first_entry = archive.by_index(0)?;
            wii_disc_info::Meta::read(&mut first_entry)?
        } else {
            let mut f = File::open(&path)?;
            wii_disc_info::Meta::read(&mut f)?
        };

        let is_disc1 = filename.contains("(Disc 1)");
        let is_disc2 = filename.contains("(Disc 2)");

        Ok(Self {
            path,
            meta,
            is_disc1,
            is_disc2,
        })
    }

    #[must_use]
    #[inline]
    fn is_multidisc(&self) -> bool {
        self.is_disc1 || self.is_disc2
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
        self.meta.game_id() == other.meta.game_id()
            && self.is_disc1 == other.is_disc1
            && self.is_disc2 == other.is_disc2
    }
}

impl Eq for ImportEntry {}

impl Hash for ImportEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.game_id().hash(state);
        self.is_disc1.hash(state);
        self.is_disc2.hash(state);
    }
}

pub fn keep_valid_games(
    games: impl IntoIterator<Item = PathBuf>,
    existing_ids: &[GameID],
) -> Vec<ImportEntry> {
    games
        .into_iter()
        .map(ImportEntry::new)
        .filter_map(Result::ok)
        .unique()
        .filter(|entry| {
            // filter out existing games; if it's a dual disc, try anyways
            entry.is_multidisc() || !existing_ids.contains(&entry.meta.game_id())
        })
        .collect()
}
