// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod pages;
mod types;
mod updater;

use app::App;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "TinyWiiBackupManager",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_zoom_factor(1.25);

            Box::<App>::default()
        }),
    )
}
