// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use tiny_wii_backup_manager::error_handling::show_error;

const WINDOW_SIZE: egui::Vec2 = egui::vec2(800.0, 600.0);

fn main() {
    let wbfs_dir = rfd::FileDialog::new()
        .set_title("Select WBFS Directory")
        .pick_folder();

    let Some(wbfs_dir) = wbfs_dir else {
        show_error("Error", "Failed to select WBFS directory");
        return;
    };

    let title = format!(
        "TinyWiiBackupManager v{} - {}",
        env!("CARGO_PKG_VERSION"),
        wbfs_dir.display()
    );

    let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../logo@2x.png")) else {
        show_error("Error", "Failed to load icon");
        return;
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_icon(icon),
        ..Default::default()
    };

    let res = eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(tiny_wii_backup_manager::App::new(cc, wbfs_dir)))
        }),
    );

    if let Err(e) = res {
        show_error("Error", &format!("Failed to run application: {e}"));
    }
}
