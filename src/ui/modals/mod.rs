// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game::Game;
use std::{path::PathBuf, sync::Arc};

pub mod delete_dir;
pub mod game_info;
pub mod homebrew_app_info;

#[derive(Debug, Clone)]
pub enum Modal {
    GameInfo((Arc<Game>, Option<wii_disc_info::Meta>)),
    HomebrewAppInfo(usize),
    DeleteDir(PathBuf),
}
