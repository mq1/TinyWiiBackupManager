// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use tiny_wii_backup_manager::PRODUCT_NAME;
use tiny_wii_backup_manager::app::App;
use tracing_core::{Level, LevelFilter};

const LOGO: &[u8] = include_bytes!("../logo-small.png");

fn main() -> eframe::Result<()> {
    let level = match std::env::var("RUST_LOG")
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "error" => Level::ERROR,
        "warn" => Level::WARN,
        "info" => Level::INFO,
        "debug" => Level::DEBUG,
        "trace" => Level::TRACE,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::from_level(level))
        .init();

    let icon = eframe::icon_data::from_png_bytes(LOGO).expect("Failed to load icon");
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(782.0, 600.0))
        .with_icon(icon);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        PRODUCT_NAME,
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
