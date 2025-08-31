// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use tiny_wii_backup_manager::PRODUCT_NAME;
use tiny_wii_backup_manager::app::App;
use tiny_wii_backup_manager::fmtlayer::{TermcolorFmtLayer, parse_level_from_str};
use tracing_core::LevelFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

const LOGO: &[u8] = include_bytes!("../logo-small.png");

fn main() -> eframe::Result<()> {
    // --- Custom Level Filtering (No `regex`) ---
    // Read the log level from the RUST_LOG environment variable.
    let log_level_str = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let min_level = parse_level_from_str(&log_level_str)
        .expect("Invalid RUST_LOG level. Use: trace, debug, info, warn, error.");

    // Create a LevelFilter based on the parsed level.
    // This is a simple filter that does NOT use regex.
    let level_filter = LevelFilter::from_level(min_level);
    // ------------------------------------------

    // Combine our `LevelFilter` with the `TermcolorFmtLayer`.
    let subscriber = Registry::default()
        .with(level_filter) // First apply the filter
        .with(TermcolorFmtLayer); // Then format the allowed events

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Log some messages to see it in action.
    tracing::trace!("This is a trace message.");
    tracing::debug!("This is a debug message.");
    tracing::info!("Hello, world! This is an info message.");
    tracing::warn!("Something to be aware of.");
    tracing::error!("An error has occurred!");
    tracing::info!(
        count = 10,
        "Multiple fields but only message is printed by current visitor."
    );

    let icon = eframe::icon_data::from_png_bytes(LOGO).expect("Failed to load icon");
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(782.0, 600.0))
        .with_icon(icon);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        PRODUCT_NAME,
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
