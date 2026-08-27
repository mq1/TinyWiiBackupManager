// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod about;
pub mod game_grid;
pub mod game_table;
pub mod homebrew_app_grid;
pub mod homebrew_app_table;
pub mod import_queue;
pub mod settings;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Games,
    HomebrewApps,
    Settings,
    Toolbox,
    ImportQueue,
    About,
}
