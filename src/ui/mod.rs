// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod choose_archive_path_dialog;
mod choose_file_to_push_dialog;
mod choose_games_dialog;
mod choose_hbc_apps_dialog;
mod choose_mount_point_dialog;
mod developers;
mod game_info;
mod games;
mod games_grid;
mod games_list;
mod hbc_app_info;
mod hbc_apps;
mod hbc_apps_grid;
mod hbc_apps_list;
mod nav;
mod osc;
mod osc_grid;
mod osc_list;
mod remove_game;
mod remove_hbc_app;
pub mod root;
mod settings;
mod status;
mod tools;

#[derive(PartialEq)]
pub enum View {
    Games,
    HbcApps,
    Osc,
    Tools,
    Settings,
}
