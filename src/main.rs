// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::types::app::App;

mod wbfs_file;
mod types;
mod updater;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "TinyWiiBackupManager",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}
