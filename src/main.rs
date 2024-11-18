// SPDX-FileCopyrightText: 2024 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod pages;
mod types;
mod updater;

use app::App;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "TinyWiiBackupManager",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::<App>::default())
        }),
    )
}
