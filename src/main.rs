// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result, anyhow};
use eframe::egui;
use tiny_wii_backup_manager::App;
use tiny_wii_backup_manager::error_handling::show_anyhow_error;

const LOGO: &[u8] = include_bytes!("../assets/linux/256x256/tiny-wii-backup-manager.png");

fn main() -> Result<()> {
    run().map_err(|e| {
        show_anyhow_error("Fatal Error", &e);
        e
    })
}

fn run() -> Result<()> {
    let wbfs_dir = rfd::FileDialog::new()
        .set_title("Select WBFS Directory")
        .pick_folder()
        .context("Failed to pick WBFS directory")?;

    let title = format!(
        "TWBM v{} - {}",
        env!("CARGO_PKG_VERSION"),
        wbfs_dir.display()
    );

    let icon = eframe::icon_data::from_png_bytes(LOGO).context("Failed to load icon")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(&title)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            match App::new(cc, wbfs_dir) {
                Ok(app) => Ok(Box::new(app) as Box<dyn eframe::App>),
                Err(e) => Err(e.into()),
            }
        }),
    )
    .map_err(|e| anyhow!("eframe error: {e}"))
}
