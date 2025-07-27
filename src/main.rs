// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{anyhow, Context, Result};
use eframe::egui;
use tiny_wii_backup_manager::error_handling::show_anyhow_error;

const WINDOW_SIZE: egui::Vec2 = egui::vec2(800.0, 600.0);

fn main() -> Result<()> {
    if let Err(e) = run() {
        show_anyhow_error("Fatal Error", &e);
        return Err(e);
    }

    Ok(())
}

fn run() -> Result<()> {
    let wbfs_dir = rfd::FileDialog::new()
        .set_title("Select WBFS Directory")
        .pick_folder()
        .context("Failed to pick WBFS directory")?;

    let title = format!(
        "TinyWiiBackupManager v{} - {}",
        env!("CARGO_PKG_VERSION"),
        wbfs_dir.display()
    );

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../logo@2x.png"))
        .context("Failed to load icon")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(WINDOW_SIZE)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            match tiny_wii_backup_manager::App::new(cc, wbfs_dir) {
                Ok(app) => Ok(Box::new(app) as Box<dyn eframe::App>),
                Err(e) => Err(e.into()),
            }
        }),
    )
    .map_err(|e| anyhow!("eframe error: {e}"))
}
