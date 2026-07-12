// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;

pub mod card;
pub mod game_card;
pub mod game_info;
pub mod games_titlebar;
pub mod link;
pub mod my_button;
pub mod notifications;
pub mod plugin_card;
pub mod plugins_titlebar;
pub mod sidebar;
pub mod sidebar_button;

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Modal {
    GameInfo(usize),
}
