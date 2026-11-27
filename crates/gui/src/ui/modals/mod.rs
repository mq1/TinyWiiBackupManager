// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use twbm_games::game::Game;
use twbm_homebrew::app::HomebrewApp;
use twbm_osc::app::OscApp;

pub mod confirm_scub;
pub mod delete_dir;
pub mod game_info;
pub mod homebrew_app_info;
pub mod osc_app_info;

#[derive(Debug, Clone)]
pub enum Modal {
    GameInfo((Game, Option<wii_disc_info::Meta>, bool)),
    HomebrewAppInfo(HomebrewApp),
    OscAppInfo(OscApp),
    DeleteDir(PathBuf),
    ConfirmScrub(Game),
}
