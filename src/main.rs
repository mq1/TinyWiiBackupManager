// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use TinyWiiBackupManager::app::App;
use TinyWiiBackupManager::util::wiitdb::WIITDB;
use eframe::egui;

const LOGO: &[u8] = include_bytes!("../assets/logo-small.png");

fn main() {
    // pre-decompress WIITDB
    std::thread::spawn(|| {
        let _ = &*WIITDB;
    });

    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    let icon = eframe::icon_data::from_png_bytes(LOGO).expect("Failed to load icon");
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(800.0, 600.0))
        .with_icon(icon);

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
