// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result, anyhow};
use eframe::egui;
use tiny_wii_backup_manager::App;
use tiny_wii_backup_manager::PRODUCT_NAME;
use tiny_wii_backup_manager::error_handling::show_anyhow_error;

const LOGO: &[u8] = include_bytes!("../assets/linux/256x256/tiny-wii-backup-manager.png");

fn main() {
    if let Err(e) = run() {
        show_anyhow_error("Fatal Error", &e);
    }
}

fn run() -> Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let mut base_dir = rfd::FileDialog::new()
        .set_title("Select base directory (usually the root of your drive)")
        .pick_folder()
        .context("Failed to pick base directory")?;

    // correct base_dir if the user has picked either "wbfs" or "games" dir
    // we only have to get the parent dir if this is the case
    // Note: file_name() returns None for drive roots on Windows (e.g., C:\, D:\, etc.)
    if let Some(dir_name) = base_dir.file_name().and_then(|n| n.to_str()) {
        if dir_name == "wbfs" || dir_name == "games" {
            base_dir = base_dir
                .parent()
                .context("Failed to get parent directory")?
                .to_path_buf();
        }
    }
    // If file_name() returns None (e.g., drive root on Windows), we just use the selected path as-is

    let title = format!("{PRODUCT_NAME}: {}", base_dir.display());

    let icon = eframe::icon_data::from_png_bytes(LOGO).context("Failed to load icon")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(782.0, 600.0))
            .with_title(&title)
            .with_icon(icon),
        ..Default::default()
    };

    // Check if updates should be disabled
    let updates_enabled = std::env::var_os("TWBM_DISABLE_UPDATES").is_none();

    eframe::run_native(
        &title,
        options,
        Box::new(move |cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            let app = App::new(cc, base_dir, updates_enabled)?;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow!(e.to_string()))
    .context("Failed to run app")
}
