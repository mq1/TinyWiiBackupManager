// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod data_dir;
mod extensions;
mod game;
mod game_id;
mod hbc;
mod http_util;
mod message;
mod notifications;
mod osc;
mod state;
mod ui;
mod util;
mod wiitdb;

use crate::{notifications::notifications_subscription, state::State};
use iced::{Size, window};
use std::sync::LazyLock;

pub static APP_ICON: LazyLock<Vec<u8>> = LazyLock::new(|| {
    image::load_from_memory_with_format(
        include_bytes!("../assets/TinyWiiBackupManager.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to load app icon")
    .into_rgba8()
    .into_vec()
});

fn main() -> iced::Result {
    #[cfg(target_os = "macos")]
    let icon = None;

    #[cfg(not(target_os = "macos"))]
    let icon = Some(
        window::icon::from_rgba(APP_ICON.clone(), 512, 512).expect("Failed to create window icon"),
    );

    let window = window::Settings {
        size: Size::new(800.0, 600.0),
        min_size: Some(Size::new(800.0, 600.0)),
        icon,
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
