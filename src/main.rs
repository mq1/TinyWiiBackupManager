// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod config;
mod convert;
mod covers;
mod disc_info;
mod extensions;
mod games;
mod hbc_apps;
mod http;
mod overflow_reader;
mod tasks;
mod titles;
mod ui;
mod updater;
mod util;

use crate::{app::App, config::Config};
use anyhow::{Result, anyhow};
use eframe::{
    NativeOptions,
    egui::{self, ViewportBuilder},
};
use egui_extras::install_image_loaders;
use std::{fs, path::PathBuf};

#[cfg(not(feature = "app-dir"))]
fn get_data_dir() -> Result<PathBuf> {
    // For portable builds, use a directory next to the executable
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path
        .parent()
        .ok_or(anyhow!("Could not get executable directory"))?;
    let data_dir = exe_dir.join("TinyWiiBackupManager-data");
    Ok(data_dir)
}

#[cfg(feature = "app-dir")]
fn get_data_dir() -> Result<PathBuf> {
    // For standard builds, use the system's app data directory
    let proj = directories::ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;
    Ok(proj.data_dir().to_path_buf())
}

fn main() -> Result<()> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;
    let mut app = App::new(&data_dir);

    // ----------------
    // Initialize tasks

    app.spawn_get_titles_task();
    app.spawn_get_games_task();
    app.spawn_check_update_task();

    // -------------
    // Initialize UI

    #[cfg(not(target_os = "macos"))]
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/logo-small.png"))
        .expect("Failed to load icon");

    #[cfg(target_os = "macos")]
    let icon = egui::IconData::default();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800., 600.])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_simple_native(env!("CARGO_PKG_NAME"), native_options, move |ctx, _| {
        install_image_loaders(ctx);
        ui::root::update(ctx, &mut app);
        app.apply_pending(ctx);
    })
    .expect("Failed to run app");

    Ok(())
}
