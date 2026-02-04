// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod data_dir;
mod games;
mod hbc;
mod http_util;
mod known_mount_points;
mod message;
mod notifications;
mod state;
mod ui;
mod updater;
mod util;

use crate::state::State;
use iced::{Size, window};
use std::env;

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[inline]
fn get_window_icon() -> Option<window::Icon> {
    let rgba8_bytes = image::load_from_memory_with_format(
        include_bytes!("../package/windows/TinyWiiBackupManager-64x64.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to load app icon")
    .into_rgba8()
    .into_vec();

    let icon = window::icon::from_rgba(rgba8_bytes, 64, 64).expect("Failed to create window icon");

    Some(icon)
}

fn main() -> iced::Result {
    unsafe {
        env::set_var("WGPU_POWER_PREF", "none");
    }

    let window = window::Settings {
        size: Size::new(800.0, 600.0),
        min_size: Some(Size::new(800.0, 600.0)),

        // x11 and windows only
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        icon: get_window_icon(),

        // wayland only
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "it.mq1.TinyWiiBackupManager".to_string(),
            ..Default::default()
        },

        ..Default::default()
    };

    let settings = iced::Settings {
        default_text_size: 14.into(),
        ..Default::default()
    };

    let app = iced::application(State::new, State::update, ui::view)
        .window(window)
        .settings(settings)
        .title(State::title)
        .theme(State::theme)
        .subscription(State::subscription);

    app.run()
}
