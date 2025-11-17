// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{disc_info::DiscInfo, wiitdb::GameInfo};

pub mod accent;
mod confirm_conversion;
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
mod wiiload;

pub const LOGO_BYTES: &[u8] = include_bytes!("../../assets/TinyWiiBackupManager.png");

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Games,
    HbcApps,
    Osc,
    Wiiload,
    Tools,
    Settings,
}

impl View {
    pub fn title(&self) -> &'static str {
        match self {
            View::Games => "Games",
            View::HbcApps => "HBC Apps",
            View::Osc => "OSC Apps",
            View::Wiiload => "Wiiload",
            View::Tools => "Tools",
            View::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Modal {
    Info,
    GameInfo(u16, Box<Option<DiscInfo>>, Box<Option<GameInfo>>),
    HbcAppInfo(u16),
    DeleteGame(u16),
    DeleteHbcApp(u16),
    ConvertGames(Box<[DiscInfo]>),
}

#[derive(Debug)]
pub enum UiAction {
    OpenView(View),
    OpenModal(Modal),
    CloseModal,
    RunNormalizePaths,
    RunDotClean,
    ApplyFilterGames,
    ApplyFilterHbcApps,
    ApplyFilterOscApps,
    TriggerDownloadOscIcons,
    DeleteGame(u16),
    TriggerRefreshGames,
    ApplySorting,
    DeleteHbcApp(u16),
    TriggerRefreshHbcApps,
}
