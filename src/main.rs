// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod data_dir;
mod disc_util;
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

#[cfg(target_vendor = "win7")]
#[link(name = "ole32")]
unsafe extern "system" {
    pub unsafe fn CoTaskMemFree(pv: *mut std::ffi::c_void);
}

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

fn main() -> iced::Result {
    #[cfg(target_os = "linux")]
    iced::futures::executor::block_on(f16_gpu_fix());

    #[cfg(target_os = "macos")]
    let height = 600.0 + 32.0; // compensate for titlebar height on macOS

    #[cfg(not(target_os = "macos"))]
    let height = 600.0;

    let window = window::Settings {
        size: Size::new(800.0, height),
        min_size: Some(Size::new(800.0, height)),

        // x11 and windows only
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        icon: get_window_icon(),

        // wayland only
        #[cfg(target_os = "linux")]
        platform_specific: window::settings::PlatformSpecific {
            application_id: "it.mq1.TinyWiiBackupManager".to_string(),
            ..Default::default()
        },

        // macos only
        #[cfg(target_os = "macos")]
        platform_specific: window::settings::PlatformSpecific {
            titlebar_transparent: true,
            fullsize_content_view: true,
            ..Default::default()
        },

        // windows 11 only
        #[cfg(target_vendor = "pc")]
        platform_specific: window::settings::PlatformSpecific {
            corner_preference: window::settings::platform::CornerPreference::Round,
            ..Default::default()
        },

        ..Default::default()
    };

    let settings = iced::Settings {
        default_text_size: 14.into(),
        ..Default::default()
    };

    iced::application(State::new, State::update, ui::view)
        .window(window)
        .settings(settings)
        .title(State::title)
        .theme(State::theme)
        .subscription(State::subscription)
        .run()
}
