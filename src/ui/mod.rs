// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod choose_games_dialog;
mod choose_hbc_apps_dialog;
mod choose_mount_point_dialog;
mod disc_info;
mod games;
mod games_grid;
mod games_list;
mod hbc_apps;
mod hbc_apps_grid;
mod nav;
mod osc;
mod remove_game;
pub mod root;
mod settings;
mod status;

#[derive(PartialEq)]
pub enum View {
    Games,
    HbcApps,
    Osc,
    Settings,
}
