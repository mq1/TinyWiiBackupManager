// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod accent;
mod developers;
pub mod dialogs;
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
mod nod_gui;
mod osc;
mod osc_app_info;
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
    NodGui,
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
            View::NodGui => "Convert",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Modal {
    Info,
    GameInfo(u16),
    HbcAppInfo(u16),
    OscAppInfo(u16),
}
