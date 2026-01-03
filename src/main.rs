// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod data_dir;
mod extensions;
mod game;
mod game_id;
mod http;
mod message;
mod notifications;
mod state;
mod ui;
mod util;
mod wiitdb;

use std::{io::Cursor, sync::LazyLock};

use crate::{notifications::notifications_subscription, state::State};
use iced::{Size, window};
use image::{DynamicImage, codecs::png::PngDecoder};
use lucide_icons::LUCIDE_FONT_BYTES;

pub static APP_ICON: LazyLock<Box<[u8]>> = LazyLock::new(|| {
    let decoder = PngDecoder::new(Cursor::new(include_bytes!(
        "../assets/TinyWiiBackupManager.png"
    )))
    .expect("Failed to decode app icon png data");

    let img = DynamicImage::from_decoder(decoder).expect("Failed to load app icon");

    img.into_rgba8().into_vec().into_boxed_slice()
});

#[cfg(target_os = "macos")]
#[inline]
fn get_app_icon() -> Option<window::Icon> {
    None
}

#[cfg(not(target_os = "macos"))]
#[inline]
fn get_app_icon() -> Option<window::Icon> {
    let icon =
        window::icon::from_rgba(APP_ICON.to_vec(), 512, 512).expect("Failed to create window icon");

    Some(icon)
}

fn main() -> iced::Result {
    let window = window::Settings {
        size: Size::new(800.0, 600.0),
        min_size: Some(Size::new(800.0, 600.0)),
        icon: get_app_icon(),
        ..Default::default()
    };

    let settings = iced::Settings {
        default_text_size: 14.into(),
        ..Default::default()
    };

    let app = iced::application(State::new, State::update, ui::view)
        .window(window)
        .settings(settings)
        .font(LUCIDE_FONT_BYTES)
        .title(env!("CARGO_PKG_NAME"))
        .subscription(notifications_subscription);

    app.run()
}
