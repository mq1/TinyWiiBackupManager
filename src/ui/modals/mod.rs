// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

pub mod delete_dir;
pub mod game_info;
pub mod homebrew_app_info;

#[derive(Debug, Clone)]
pub enum Modal {
    GameInfo((usize, Option<wii_disc_info::Meta>)),
    HomebrewAppInfo(usize),
    DeleteDir(PathBuf),
}
