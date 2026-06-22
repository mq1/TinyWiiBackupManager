// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod drive_info;
mod executor;
mod games;
mod messages;
mod notifications;
mod plugins;
mod state;
mod ui;
mod update;
mod util;

use crate::executor::DumbExecutor;
use crate::state::AppState;
use anyhow::{Result, anyhow};
use lucide_icons::LUCIDE_FONT_BYTES;
use std::fs;

pub fn main() -> Result<()> {
    let data_dir = util::get_data_dir().ok_or_else(|| anyhow!("Unable to get data directory"))?;
    fs::create_dir_all(&data_dir)?;

    let boot = move || AppState::new(data_dir.clone());

    let settings = iced::Settings {
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        id: Some(String::from("it.mq1.TinyWiiBackupManager")),
        ..Default::default()
    };

    let window = iced::window::Settings {
        size: iced::Size::new(800., 600.),
        min_size: Some(iced::Size::new(800., 600.)),

        #[cfg(target_vendor = "pc")]
        platform_specific: window::settings::PlatformSpecific {
            corner_preference: iced::window::settings::platform::CornerPreference::Round,
            ..Default::default()
        },

        #[cfg(target_os = "macos")]
        platform_specific: iced::window::settings::PlatformSpecific {
            titlebar_transparent: true,
            ..Default::default()
        },

        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: String::from("it.mq1.TinyWiiBackupManager"),
            ..Default::default()
        },

        ..Default::default()
    };

    iced::application(boot, AppState::update, ui::root::view)
        .settings(settings)
        .window(window)
        .title(ui::title)
        .executor::<DumbExecutor>()
        .run()?;

    Ok(())
}
