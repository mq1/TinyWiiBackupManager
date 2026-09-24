// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::fs::get_dir_size;
use anyhow::{Context, Result, bail};
use arrayvec::ArrayString;
use iced::{
    advanced::image::{Allocation, allocate},
    widget::image,
};
use size::Size;
use smol::fs;
use std::fmt::Write;
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
enum GameTitle {
    FromIdMap(&'static str),
    FromDisc(Box<str>),
}

impl GameTitle {
    fn as_str(&self) -> &str {
        match self {
            GameTitle::FromIdMap(title) => title,
            GameTitle::FromDisc(title) => title,
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
    path: PathBuf,
    id: GameID,
    title: GameTitle,
    size: u64,
    size_str: ArrayString<10>,
    is_wii: bool,
    cover: Option<Allocation>,
    search_term_lowercase: Box<str>,
}

impl Game {
    pub async fn try_from_path(path: PathBuf, is_wii: bool) -> Result<Self> {
        // Check if the path is a directory
        if !fs::metadata(&path).await?.is_dir() {
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
            None => GameTitle::FromDisc(Box::from(title_raw.trim())),
        };

        let size = get_dir_size(&path).await;

        let mut size_str = ArrayString::new();
        let _ = write!(&mut size_str, "{}", Size::from_bytes(size));

        let search_term_lowercase = format!("{}\0{}", title, id).to_lowercase().into_boxed_str();

        Ok(Self {
            path,
            id,
            title,
            size,
            size_str,
            is_wii,
            cover: None,
            search_term_lowercase,
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

    pub async fn get_disc_path(&self) -> Option<PathBuf> {
        for path in self.get_possible_disc_paths() {
            if fs::metadata(&path).await.is_ok_and(|meta| meta.is_file()) {
                return Some(path);
            }
        }

        None
    }

    pub fn get_disc_path_blocking(&self) -> Option<PathBuf> {
        self.get_possible_disc_paths()
            .find(|path| std::fs::metadata(path).is_ok_and(|meta| meta.is_file()))
    }

    pub fn load_cover_blocking(&mut self, data_dir: &Path) {
        let cover_path = data_dir
            .join("covers")
            .join(self.id.as_str())
            .with_extension("png");

        self.cover = std::fs::read(cover_path)
            .ok()
            .map(image::Handle::from_bytes)
            .map(|handle| unsafe { allocate(&handle, (176, 248).into()) })
    }

    pub fn matches_search(&self, search_term: &str) -> bool {
        self.search_term_lowercase.contains(search_term)
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn size_str(&self) -> &str {
        &self.size_str
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

    pub fn cover(&self) -> Option<&image::Handle> {
        self.cover.as_ref().map(|cover| cover.handle())
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Game {}
