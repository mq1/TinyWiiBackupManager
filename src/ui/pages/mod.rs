// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod game_grid;
pub mod plugins;
pub mod settings;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Games,
    Settings,
    Toolbox,
    Plugins,
}
