// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod data_dir;
mod games;
mod hbc;
mod http_util;
mod message;
mod notifications;
mod state;
mod ui;
mod updater;
mod util;

use crate::{notifications::notifications_subscription, state::State};
use iced::{Size, window};
use std::{env, sync::LazyLock};

pub static APP_ICON: LazyLock<Vec<u8>> = LazyLock::new(|| {
    image::load_from_memory_with_format(
        include_bytes!("../assets/TinyWiiBackupManager@0.5x.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to load app icon")
    .into_rgba8()
    .into_vec()
});

fn main() -> iced::Result {
    unsafe {
        env::set_var("SMOL_THREADS", "2");
        env::set_var("WGPU_POWER_PREF", "low");
    }

    let window = window::Settings {
        size: Size::new(800.0, 600.0),
        min_size: Some(Size::new(800.0, 600.0)),

        #[cfg(any(target_os = "linux", target_os = "windows"))]
        icon: Some(
            window::icon::from_rgba(APP_ICON.clone(), 256, 256)
                .expect("Failed to create window icon"),
        ),

        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "TinyWiiBackupManager".to_string(),
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
        .subscription(notifications_subscription);

    app.run()
}
