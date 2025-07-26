// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Result, Context};

fn main() -> Result<()> {
    let Some(wbfs_dir) = rfd::FileDialog::new()
        .set_title("Select WBFS Directory")
        .pick_folder() else {
            return Ok(());
        };

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../logo@2x.png"))
        .context("Failed to load icon")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        &format!("TinyWiiBackupManager v{} - {}", 
            env!("CARGO_PKG_VERSION"),
            wbfs_dir.display()
        ),
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(tiny_wii_backup_manager::App::new(cc, wbfs_dir)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e))
}