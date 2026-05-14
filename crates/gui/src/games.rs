// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedGame, util::GIB};
use slint::{Image, ToSharedString};
use std::path::Path;
use twbm_core::{data_dir::DATA_DIR, game::Game};

impl From<&Game> for DisplayedGame {
    fn from(game: &Game) -> Self {
        let cover_path = DATA_DIR.join(format!("covers/{}.png", game.id));
        let cover = Image::load_from_path(&cover_path).unwrap_or_default();

        Self {
            id: game.id.to_shared_string(),
            title: game.title.to_shared_string(),
            path: game.path.to_string_lossy().to_shared_string(),
            size_gib: game.size as f32 / GIB,
            is_wii: game.is_wii,
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

pub fn scan_drive(root_path: &Path) -> Vec<Game> {
    let wii_games = twbm_core::game::scan_dir(&root_path.join("wbfs"));
    let gc_games = twbm_core::game::scan_dir(&root_path.join("games"));

    wii_games.into_iter().chain(gc_games).collect()
}
