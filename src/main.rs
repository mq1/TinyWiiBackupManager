// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod archive;
mod banners;
mod checksum;
mod config;
mod convert;
mod covers;
mod dir_layout;
mod disc_info;
mod extensions;
mod games;
mod hbc_apps;
mod http;
mod notifications;
mod osc;
mod overflow_reader;
mod overflow_writer;
mod tasks;
mod titles;
mod txtcodes;
mod ui;
mod updater;
mod util;
mod wiiload;
mod wiitdb;

pub mod known_mount_points;
mod messages;

use crate::app::App;
use anyhow::{Result, anyhow};
use eframe::{
    NativeOptions,
    egui::{CornerRadius, ViewportBuilder, vec2},
};
use egui_extras::install_image_loaders;
use std::{fs, path::PathBuf};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn get_data_dir() -> Result<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe()
        && exe_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.contains("portable"))
    {
        let parent = exe_path
            .parent()
            .ok_or(anyhow!("Could not get executable directory"))?;

        let data_dir = parent.join("TinyWiiBackupManager-data");

        Ok(data_dir)
    } else {
        let proj = directories::ProjectDirs::from("it", "mq1", APP_NAME)
            .ok_or(anyhow!("Failed to get project dirs"))?;

        let data_dir = proj.data_dir().to_path_buf();

        Ok(data_dir)
    }
}

fn main() -> Result<()> {
    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;
    let mut app = App::new(data_dir);

    // ----------------
    // Initialize tasks

    titles::spawn_get_titles_task(&app.task_processor, app.data_dir.clone()); // this also loads games when finished
    updater::spawn_check_update_task(&app.task_processor);
    osc::spawn_load_osc_apps_task(&app.task_processor, &app.data_dir);
    wiitdb::spawn_load_wiitdb_task(&app.task_processor, app.config.contents.mount_point.clone());

    // -------------
    // Initialize UI

    let icon = if cfg!(target_os = "macos") {
        eframe::egui::IconData::default()
    } else {
        eframe::icon_data::from_png_bytes(ui::LOGO_BYTES).expect("Failed to load icon")
    };

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800., 600.])
            .with_min_inner_size([745., 390.])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.set_theme(app.config.contents.theme_preference);

            cc.egui_ctx.all_styles_mut(|style| {
                style.visuals.selection.bg_fill = ui::accent::get_accent_color();
                style.visuals.selection.stroke.color = style.visuals.strong_text_color();

                style.visuals.widgets.active.corner_radius = CornerRadius::same(30);
                style.visuals.widgets.hovered.corner_radius = CornerRadius::same(30);
                style.visuals.widgets.inactive.corner_radius = CornerRadius::same(30);
                style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
                style.visuals.widgets.open.corner_radius = CornerRadius::same(30);

                style.spacing.button_padding = vec2(5., 2.5);
            });

            // Load hbc apps instantly
            app.refresh_hbc_apps();

            if !app.is_mount_point_known() {
                app.notifications.show_info_no_duration("New Drive detected, a path normalization run is recommended\nYou can find it in the 🔧 Tools page");
            }

            Ok(Box::new(app))
        }),
    )
    .expect("Failed to run app");

    Ok(())
}
