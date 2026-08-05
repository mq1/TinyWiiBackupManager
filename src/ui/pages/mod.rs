// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod about;
pub mod game_grid;
pub mod game_table;
pub mod homebrew_app_grid;
pub mod settings;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Games,
    HomebrewApps,
    Settings,
    Toolbox,
    About,
}
