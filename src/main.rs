// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

// Hide console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};

fn main() -> Result<()> {
    // Prompt the user to select the WBFS directory
    let Some(wbfs_dir) = rfd::FileDialog::new().set_title("Select WBFS Directory").pick_folder() else {
        return Ok(()); // Exit gracefully if no directory is selected
    };

    // Configure native options for the eframe application
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]) // Set initial window size
            .with_icon(
                // Load the application icon from embedded bytes
                eframe::icon_data::from_png_bytes(&include_bytes!("../logo@2x.png")[..])
                    .context("Failed to load icon")?,
            ),
        ..Default::default() // Use default values for other options
    };

    // Run the eframe application
    eframe::run_native(
        &format!(
            "TinyWiiBackupManager v{} - {}",
            env!("CARGO_PKG_VERSION"),
            wbfs_dir.display()
        ),
        native_options,
        Box::new(|cc| {
            // Install image loaders for egui to display game covers
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Create and return the main application instance
            Ok(Box::new(tiny_wii_backup_manager::App::new(cc, wbfs_dir)))
        }),
    )
        .map_err(|e| anyhow::anyhow!("Failed to run eframe: {}", e)) // Convert eframe::Error to anyhow::Error
}