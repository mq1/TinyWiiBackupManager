// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, SortBy, data_dir::DATA_DIR, id_map, util::GIB};
use anyhow::Result;
use slint::{Image, SharedString, ToSharedString};
use std::{cell::RefCell, cmp::Ordering, fs, path::Path, rc::Rc};

impl Game {
    #[must_use]
    pub fn maybe_from_path(path: &Path, is_wii: bool) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }

        let filename = path.file_name()?.to_str()?;
        if filename.starts_with('.') {
            return None;
        }

        let (title_str, id_str) = filename.split_once('[')?;
        let id = id_str.strip_suffix(']')?;
        if !matches!(id.len(), 4 | 6) {
            return None;
        }

        let title = match id_map::get(id) {
            Some(e) => e.title.to_shared_string(),
            None => title_str.trim().to_shared_string(),
        };

        let size = fs_extra::dir::get_size(path).ok()?;

        #[allow(clippy::cast_precision_loss)]
        let size_gib = size as f32 / GIB;

        let cover_path = DATA_DIR.join("covers").join(format!("{id}.png"));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();

        let search_term = format!("{}\0{}", title, id)
            .to_lowercase()
            .to_shared_string();

        Some(Self {
            path: path.to_string_lossy().to_shared_string(),
            is_wii,
            size_gib,
            title,
            id: id.to_shared_string(),
            cover,
            search_term,
        })
    }

    pub fn reload_cover(&mut self) {
        let cover_path = DATA_DIR.join("covers").join(format!("{}.png", self.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        self.cover = cover;
    }
}

pub fn get_compare_fn(sort_by: Rc<RefCell<SortBy>>) -> Box<dyn Fn(&Game, &Game) -> Ordering> {
    Box::new(move |a, b| {
        let sort_by = sort_by.borrow();
        match *sort_by {
            SortBy::NameAscending => a.title.cmp(&b.title),
            SortBy::NameDescending => b.title.cmp(&a.title),
            SortBy::SizeAscending => a.size_gib.total_cmp(&b.size_gib),
            SortBy::SizeDescending => b.size_gib.total_cmp(&a.size_gib),
        }
    })
}

pub fn get_filter_fn(query_lowercase: Rc<RefCell<SharedString>>) -> Box<dyn Fn(&Game) -> bool> {
    Box::new(move |game| {
        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        game.search_term.contains(query_lowercase.as_str())
    })
}

fn scan_dir(dir: &Path, is_wii: bool, games: &mut Vec<Game>) -> Result<()> {
    let entries = fs::read_dir(dir)?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if let Some(game) = Game::maybe_from_path(&path, is_wii) {
            games.push(game);
        }
    }

    Ok(())
}

pub fn scan_drive(root_path: &Path) -> Vec<Game> {
    let mut games = Vec::new();

    let _ = scan_dir(&root_path.join("wbfs"), true, &mut games);
    let _ = scan_dir(&root_path.join("games"), false, &mut games);

    games
}
