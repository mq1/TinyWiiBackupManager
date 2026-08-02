// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod card;
pub mod game_card;
pub mod game_info;
pub mod game_row;
pub mod games_titlebar;
pub mod homebrew_app_card;
pub mod homebrew_apps_titlebar;
pub mod link;
pub mod my_button;
pub mod notifications;
pub mod sidebar;
pub mod sidebar_button;
pub mod view_as;

#[derive(Debug, Clone)]
pub enum Modal {
    GameInfo((usize, Option<Box<wii_disc_info::Meta>>)),
}
