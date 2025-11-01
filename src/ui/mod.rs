// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(feature = "accent")]
pub mod accent;

mod choose_archive_path_dialog;
mod choose_file_to_push_dialog;
mod choose_games_dialog;
mod choose_hbc_apps_dialog;
mod choose_mount_point_dialog;
mod delete_game;
mod delete_hbc_app;
mod developers;
mod game_info;
mod games;
mod games_grid;
mod games_list;
mod hbc_app_info;
mod hbc_apps;
mod hbc_apps_grid;
mod hbc_apps_list;
mod info;
mod nav;
mod osc;
mod osc_grid;
mod osc_list;
pub mod root;
mod settings;
mod status;
mod tools;

pub const LOGO_BYTES: &[u8] = include_bytes!("../../assets/TinyWiiBackupManager.png");

#[derive(PartialEq)]
pub enum View {
    Games,
    HbcApps,
    Osc,
    Tools,
    Settings,
}
