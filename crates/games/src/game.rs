// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Context, Result, bail};
use size::Size;
use std::fs::{self, File};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use twbm_common::util::make_search_term;
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
enum GameTitle {
    FromIdMap(&'static str),
    FromFileName(Box<str>),
}

impl GameTitle {
    fn as_str(&self) -> &str {
        match self {
            GameTitle::FromIdMap(title) => title,
            GameTitle::FromFileName(title) => title,
        }
    }
}

impl std::fmt::Display for GameTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    path: Box<Path>,
    id: GameID,
    title: GameTitle,
    size: Option<Size>,
    is_wii: bool,
    cover: Option<Box<Path>>,
    search_term_lowercase: Box<str>,
    gametdb_url: Box<str>,
}

impl Game {
    pub fn try_from_path(
        path: impl Into<Box<Path>>,
        is_wii: bool,
        data_dir: &Path,
    ) -> Result<Self> {
        let path = path.into();

        // Check if the path is a directory
        if !fs::metadata(&path)?.is_dir() {
            bail!("{} is not a directory", path.display());
        }

        // Get the directory name
        let dir_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .context("invalid filename")?;

        if dir_name.starts_with('.') {
            bail!("hidden directory");
        }

        // Extract title and id from the directory name
        let (title_raw, id_raw) = dir_name.split_once('[').context("invalid filename")?;

        let id_raw = id_raw.strip_suffix(']').context("invalid filename")?;

        // Parse the id
        let id = id_raw.parse::<GameID>().context("invalid game id")?;

        // get the pretty title
        let title = match twbm_idmap::get_title(id) {
            Some(title) => GameTitle::FromIdMap(title),
            None => GameTitle::FromFileName(title_raw.trim().into()),
        };

        let size = fs_extra::dir::get_size(&path).ok().map(Size::from_bytes);

        let search_term_lowercase = make_search_term(&[title.as_str(), id.as_str()]);

        let gametdb_url = format!("https://www.gametdb.com/Wii/{id}").into_boxed_str();

        let cover_path = data_dir
            .join("covers")
            .join(id.as_str())
            .with_extension("png");

        let cover = fs::exists(&cover_path)
            .unwrap_or(false)
            .then(|| cover_path.into());

        Ok(Self {
            path,
            id,
            title,
            size,
            is_wii,
            cover,
            search_term_lowercase,
            gametdb_url,
        })
    }

    fn get_possible_disc_paths<'a>(&'a self) -> impl Iterator<Item = PathBuf> + 'a {
        [
            format!("{}.wbfs", self.id),
            format!("{}.iso", self.id),
            format!("{}.part0.iso", self.id),
            "game.iso".to_string(),
            "game.ciso".to_string(),
        ]
        .into_iter()
        .map(|filename| self.path.join(filename))
    }

    pub fn get_disc_path(&self) -> Option<PathBuf> {
        self.get_possible_disc_paths()
            .find(|path| fs::metadata(path).is_ok_and(|meta| meta.is_file()))
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn size(&self) -> Option<Size> {
        self.size
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn id(&self) -> GameID {
        self.id
    }

    pub fn id_str(&self) -> &str {
        self.id.as_str()
    }

    pub fn is_wii(&self) -> bool {
        self.is_wii
    }

    pub fn is_ngc(&self) -> bool {
        !self.is_wii
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cover(&self) -> Option<&Path> {
        self.cover.as_deref()
    }

    pub fn gametdb_url(&self) -> &str {
        &self.gametdb_url
    }

    pub fn reset_cover_path(&mut self, data_dir: &Path) {
        let cover_path = data_dir
            .join("covers")
            .join(self.id.as_str())
            .with_extension("png");

        self.cover = fs::exists(&cover_path)
            .unwrap_or(false)
            .then(|| cover_path.into_boxed_path())
    }

    pub fn get_disc_info(&self) -> Result<wii_disc_info::Meta> {
        let disc_path = self.get_disc_path().context("Disc not found")?;
        let mut file = File::open(&disc_path)?;
        let meta = wii_disc_info::Meta::read(&mut file)?;
        Ok(meta)
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
