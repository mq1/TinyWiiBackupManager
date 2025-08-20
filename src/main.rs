// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result, anyhow};
use eframe::egui;
use tiny_wii_backup_manager::App;
use tiny_wii_backup_manager::error_handling::show_anyhow_error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;

const LOGO: &[u8] = include_bytes!("../assets/linux/256x256/tiny-wii-backup-manager.png");

fn main() {
    if let Err(e) = run() {
        show_anyhow_error("Fatal Error", &e);
    }
}

fn run() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                // Default to info level
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy()
                // This module is noisy at info level
                .add_directive("wgpu_core::device::resource=warn".parse()?),
        )
        .init();

    let mut base_dir = rfd::FileDialog::new()
        .set_title("Select base directory (usually the root of your drive)")
        .pick_folder()
        .context("Failed to pick base directory")?;

    let dir_name = base_dir
        .file_name()
        .and_then(|n| n.to_str())
        .context("Failed to get dir name")?;

    // correct base_dir if the user has picked either "wbfs" or "games" dir
    // we only have to get the parent dir if this is the case
    if dir_name == "wbfs" || dir_name == "games" {
        base_dir = base_dir
            .parent()
            .context("Failed to get parent directory")?
            .to_path_buf();
    }

    let title = format!(
        "TWBM v{} - {}",
        env!("CARGO_PKG_VERSION"),
        base_dir.display()
    );

    let icon = eframe::icon_data::from_png_bytes(LOGO).context("Failed to load icon")?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(782.0, 600.0))
            .with_title(&title)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            match App::new(cc, base_dir) {
                Ok(app) => Ok(Box::new(app) as Box<dyn eframe::App>),
                Err(e) => Err(e.into()),
            }
        }),
    )
    .map_err(|e| anyhow!("eframe error: {e}"))
}
