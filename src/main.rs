// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Don't show windows terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod games;
pub mod hbc_apps;
pub mod http;
pub mod wiitdb;

use anyhow::{Error, Result, anyhow};
use rfd::{MessageDialog, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel};
use std::rc::Rc;

slint::include_modules!();

fn show_err(e: &Error) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Error")
        .set_description(e.to_string())
        .show();
}

fn refresh_dir_name(handle: &MainWindow) {
    let config = config::get();
    let dir_name = config
        .mount_point
        .file_name()
        .unwrap_or(config.mount_point.as_os_str())
        .to_str()
        .unwrap_or_default();

    handle.set_dir_name(dir_name.to_shared_string());
}

fn refresh_games(handle: &MainWindow) {
    let games_res = games::list();

    if let Ok(games) = games_res {
        handle.set_games(ModelRc::from(Rc::new(VecModel::from(games))));
    } else if let Err(e) = games_res {
        show_err(&e.context("Failed to list games"));
    }
}

fn refresh_hbc_apps(handle: &MainWindow) {
    let hbc_apps_res = hbc_apps::list();

    if let Ok(hbc_apps) = hbc_apps_res {
        handle.set_hbc_apps(ModelRc::from(Rc::new(VecModel::from(hbc_apps))));
    } else if let Err(e) = hbc_apps_res {
        show_err(&e.context("Failed to list hbc apps"));
    }
}

fn run() -> Result<()> {
    let app = MainWindow::new()?;

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    refresh_dir_name(&app);
    refresh_games(&app);
    refresh_hbc_apps(&app);

    let weak = app.as_weak();
    app.on_choose_mount_point(move || {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            if let Err(e) = config::update(|config| {
                config.mount_point.clone_from(&dir);
            }) {
                show_err(&e);
            }

            if let Some(handle) = weak.upgrade() {
                refresh_dir_name(&handle);
                refresh_games(&handle);
                refresh_hbc_apps(&handle);
            } else {
                show_err(&anyhow!("Failed to upgrade weak reference"));
            }
        }
    });

    app.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run().map_err(|e| {
        show_err(&e);
        e
    })
}
