// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
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
        ..Default::default()
    };

    let window = iced::window::Settings {
        size: iced::Size::new(800., 600.),
        min_size: Some(iced::Size::new(800., 600.)),

        #[cfg(target_os = "macos")]
        platform_specific: iced::window::settings::PlatformSpecific {
            titlebar_transparent: true,
            ..Default::default()
        },

        ..Default::default()
    };

    iced::application(boot, AppState::update, ui::root::view)
        .settings(settings)
        .window(window)
        .executor::<DumbExecutor>()
        .run()?;

    Ok(())
}
