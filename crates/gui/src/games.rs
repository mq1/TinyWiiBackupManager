// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedGame, util::GIB};
use slint::{Image, SharedString, ToSharedString};
use std::{cell::RefCell, cmp::Ordering, path::Path, rc::Rc};
use twbm_core::{config::SortBy, data_dir::DATA_DIR, game::Game};

impl From<&Game> for DisplayedGame {
    fn from(game: &Game) -> Self {
        let cover_path = DATA_DIR.join(format!("covers/{}.png", game.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        let search_term = format!("{}\0{}", game.title, game.id).to_lowercase();

        Self {
            uid: game.uid,
            id: game.id.to_shared_string(),
            title: game.title.to_shared_string(),
            path: game.path.to_string_lossy().to_shared_string(),
            size_gib: game.size as f32 / GIB,
            is_wii: game.is_wii,
            search_term: search_term.to_shared_string(),
            cover,
        }
    }
}

impl DisplayedGame {
    pub fn reload_cover(&mut self) {
        let cover_path = DATA_DIR.join(format!("covers/{}.png", self.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();
        self.cover = cover;
    }
}

pub fn get_compare_fn(
    config: Rc<RefCell<SortBy>>,
) -> impl Fn(&DisplayedGame, &DisplayedGame) -> Ordering + 'static {
    move |a, b| match *config.borrow() {
        SortBy::NameDescending => a.title.cmp(&b.title),
        SortBy::NameAscending => b.title.cmp(&a.title),
        SortBy::SizeDescending => a.size_gib.total_cmp(&b.size_gib),
        SortBy::SizeAscending => b.size_gib.total_cmp(&a.size_gib),
    }
}

pub fn get_filter_fn(
    query_lowercase: Rc<RefCell<SharedString>>,
    show_wii: Rc<RefCell<bool>>,
    show_gc: Rc<RefCell<bool>>,
) -> impl Fn(&DisplayedGame) -> bool {
    move |game| {
        if !*show_wii.borrow() && game.is_wii {
            return false;
        }

        if !*show_gc.borrow() && !game.is_wii {
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
