// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, homebrew::homebrew_app::HomebrewApp, osc::osc_app::OscApp};
use std::path::PathBuf;

pub mod delete_dir;
pub mod game_info;
pub mod homebrew_app_info;
pub mod osc_app_info;

#[derive(Debug, Clone)]
pub enum Modal {
    GameInfo((Game, Option<wii_disc_info::Meta>)),
    HomebrewAppInfo(HomebrewApp),
    OscAppInfo(OscApp),
    DeleteDir(PathBuf),
}
