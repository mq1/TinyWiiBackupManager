// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedGame, data_dir::DATA_DIR, util::GIB};
use slint::{Image, SharedString, ToSharedString};
use std::{cell::RefCell, cmp::Ordering, path::Path, rc::Rc};
use twbm_core::{
    config::{Config, SortBy},
    game::Game,
};

impl DisplayedGame {
    pub fn new(game: &Game, idx: usize) -> Self {
        let cover_path = DATA_DIR.join(format!("covers/{}.png", game.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        let search_term = format!("{}\0{}", game.title, game.id).to_lowercase();

        Self {
            id: game.id.to_shared_string(),
            title: game.title.to_shared_string(),
            path: game.path.to_string_lossy().to_shared_string(),
            size_gib: game.size as f32 / GIB,
            is_wii: game.is_wii,
            search_term: search_term.to_shared_string(),
            cover,
            idx: idx as i32,
        }
    }

    pub fn reload_cover(&mut self) {
        let cover_path = DATA_DIR.join(format!("covers/{}.png", self.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        self.cover = cover;
    }
}

pub fn get_compare_fn(
    config: Rc<RefCell<Config>>,
) -> impl Fn(&DisplayedGame, &DisplayedGame) -> Ordering {
    move |a, b| {
        let config = config.borrow();

        match config.contents.sort_by {
            SortBy::NameDescending => a.title.cmp(&b.title),
            SortBy::NameAscending => b.title.cmp(&a.title),
            SortBy::SizeDescending => a.size_gib.total_cmp(&b.size_gib),
            SortBy::SizeAscending => b.size_gib.total_cmp(&a.size_gib),
        }
    }
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
    config: Rc<RefCell<Config>>,
) -> impl Fn(&DisplayedGame) -> bool {
    move |game| {
        let config = config.borrow();

        if !config.contents.show_wii && game.is_wii {
            return false;
        }

        if !config.contents.show_gc && !game.is_wii {
            return false;
        }

        let query_lowercase = query_lowercase.borrow();

        if query_lowercase.is_empty() {
            return true;
        }

        game.search_term.contains(query_lowercase.as_str())
    }
}

pub fn scan_drive(root_path: &Path) -> Vec<Game> {
    let wii_games = twbm_core::game::scan_dir(&root_path.join("wbfs"));
    let gc_games = twbm_core::game::scan_dir(&root_path.join("games"));

    wii_games.into_iter().chain(gc_games).collect()
}
