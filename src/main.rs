// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// lints
#![warn(clippy::all, rust_2018_idioms)]
#![allow(unstable_name_collisions)]
//
// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod errors;
mod games;
mod homebrew;
mod messages;
mod notifications;
mod state;
mod ui;
mod update;
mod util;

use crate::state::AppState;
use lucide_icons::LUCIDE_FONT_BYTES;

pub fn main() -> iced::Result {
    unsafe {
        std::env::set_var("SMOL_THREADS", "1");
        std::env::set_var("BLOCKING_MAX_THREADS", "10");
    }

    let data_dir = util::data_dir::get_data_dir().expect("Unable to get data directory");

    let settings = iced::Settings {
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        default_text_size: 14.into(),
        id: Some(String::from("it.mq1.TinyWiiBackupManager")),
        ..Default::default()
    };

    #[cfg(target_os = "macos")]
    let height = 600.0 + 32.0; // compensate for titlebar height on macOS

    #[cfg(not(target_os = "macos"))]
    let height = 600.0;

    let window = iced::window::Settings {
        size: iced::Size::new(800., height),
        min_size: Some(iced::Size::new(800., height)),

        #[cfg(target_os = "windows")]
        platform_specific: iced::window::settings::PlatformSpecific {
            corner_preference: iced::window::settings::platform::CornerPreference::Round,
            ..Default::default()
        },

        #[cfg(target_os = "macos")]
        platform_specific: iced::window::settings::PlatformSpecific {
            titlebar_transparent: true,
            fullsize_content_view: true,
            ..Default::default()
        },

        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: String::from("it.mq1.TinyWiiBackupManager"),
            ..Default::default()
        },

        ..Default::default()
    };

    iced::application(AppState::boot(data_dir), AppState::update, ui::root::view)
        .settings(settings)
        .window(window)
        .title(ui::title)
        .run()
}
