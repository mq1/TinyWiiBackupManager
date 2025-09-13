// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use tiny_wii_backup_manager::PRODUCT_NAME;
use tiny_wii_backup_manager::app::App;

const LOGO: &[u8] = include_bytes!("../assets/logo-small.png");

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let icon = eframe::icon_data::from_png_bytes(LOGO).expect("Failed to load icon");
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(800.0, 600.0))
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
