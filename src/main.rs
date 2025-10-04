// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use TinyWiiBackupManager::app::App;
use TinyWiiBackupManager::util::wiitdb::WIITDB;
use eframe::egui::{self, ViewportBuilder};
use std::sync::Arc;

fn main() {
    // pre-decompress WIITDB
    std::thread::spawn(|| {
        let _ = &*WIITDB;
    });

    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    #[cfg(not(target_os = "macos"))]
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/logo-small.png"))
        .expect("Failed to load icon");

    #[cfg(target_os = "macos")]
    let icon = egui::IconData::default();

    let viewport = ViewportBuilder {
        inner_size: Some(egui::vec2(800.0, 600.0)),
        icon: Some(Arc::new(icon)),
        ..Default::default()
    };

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        env!("CARGO_PKG_NAME"),
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    ) {
        let _ = rfd::MessageDialog::new()
            .set_title("Error")
            .set_description(format!("Error: {e:?}"))
            .set_level(rfd::MessageLevel::Error)
            .show();

        std::process::exit(1);
    }
}
