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

#[cfg(any(target_os = "windows", target_os = "linux"))]
#[inline]
fn get_window_icon() -> Option<iced::window::Icon> {
    let rgba8_bytes = image::load_from_memory_with_format(
        include_bytes!("../package/windows/TinyWiiBackupManager-64x64.png"),
        image::ImageFormat::Png,
    )
    .expect("Failed to load app icon")
    .into_rgba8()
    .into_vec();

    let icon =
        iced::window::icon::from_rgba(rgba8_bytes, 64, 64).expect("Failed to create window icon");

    Some(icon)
}

#[cfg(target_os = "linux")]
async fn f16_gpu_fix() {
    let instance = wgpu::Instance::default();

    let adapter_options = wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::from_env()
            .unwrap_or(wgpu::PowerPreference::HighPerformance),
        compatible_surface: None,
        force_fallback_adapter: false,
    };

    let Ok(adapter) = instance.request_adapter(&adapter_options).await else {
        return;
    };

    if !adapter.features().contains(wgpu::Features::SHADER_F16) {
        unsafe {
            std::env::set_var("ICED_BACKEND", "tiny-skia");
        }
    }
}

pub fn main() -> iced::Result {
    unsafe {
        std::env::set_var("SMOL_THREADS", "1");
        std::env::set_var("BLOCKING_MAX_THREADS", "10");
    }

    #[cfg(target_os = "linux")]
    smol::block_on(f16_gpu_fix());

    let settings = iced::Settings {
        fonts: vec![LUCIDE_FONT_BYTES.into()],
        default_text_size: 14.into(),
        id: Some("it.mq1.TinyWiiBackupManager".into()),
        ..Default::default()
    };

    #[cfg(target_os = "macos")]
    let height = 600.0 + 32.0; // compensate for titlebar height on macOS

    #[cfg(not(target_os = "macos"))]
    let height = 600.0;

    let window = iced::window::Settings {
        size: iced::Size::new(800., height),
        min_size: Some(iced::Size::new(800., height)),

        // linux x11 and windows only
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        icon: get_window_icon(),

        // windows 11 only
        #[cfg(target_os = "windows")]
        platform_specific: iced::window::settings::PlatformSpecific {
            corner_preference: iced::window::settings::platform::CornerPreference::Round,
            ..Default::default()
        },

        // macOS only
        #[cfg(target_os = "macos")]
        platform_specific: iced::window::settings::PlatformSpecific {
            titlebar_transparent: true,
            fullsize_content_view: true,
            ..Default::default()
        },

        // linux wayland only
        #[cfg(target_os = "linux")]
        platform_specific: iced::window::settings::PlatformSpecific {
            application_id: "it.mq1.TinyWiiBackupManager".into(),
            ..Default::default()
        },

        ..Default::default()
    };

    iced::application(AppState::boot, AppState::update, ui::root::view)
        .settings(settings)
        .window(window)
        .title(ui::title)
        .subscription(AppState::subscription)
        .run()
}
