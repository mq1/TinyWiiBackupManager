// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Don't show windows terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod add_games;
pub mod concurrency;
pub mod config;
pub mod fs;
pub mod games;
pub mod hbc_apps;
pub mod http;
pub mod tasks;
pub mod titles;
pub mod updater;
pub mod watcher;
pub mod wiitdb;

use crate::fs::get_disk_usage;
use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel};
use std::{fmt::Display, rc::Rc, sync::OnceLock};

slint::include_modules!();

pub static PROJ: OnceLock<ProjectDirs> = OnceLock::new();

fn show_err(e: impl Display) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Error")
        .set_description(e.to_string())
        .show();
}

fn refresh_dir_name(handle: &MainWindow) {
    let mount_point = config::get().mount_point;

    let dir_name = mount_point
        .file_name()
        .unwrap_or(mount_point.as_os_str())
        .to_str()
        .unwrap_or_default();

    handle.set_dir_name(dir_name.to_shared_string());
}

fn refresh_disk_usage(handle: &MainWindow) {
    let path = config::get().mount_point;

    let usage = get_disk_usage(&path);
    handle.set_disk_usage(usage.to_shared_string());
}

fn refresh_games(handle: &MainWindow) -> Result<()> {
    let games = games::list()?;
    handle.set_games(ModelRc::from(Rc::new(VecModel::from(games))));
    Ok(())
}

fn refresh_hbc_apps(handle: &MainWindow) -> Result<()> {
    let hbc_apps = hbc_apps::list()?;
    handle.set_hbc_apps(ModelRc::from(Rc::new(VecModel::from(hbc_apps))));
    Ok(())
}

fn run() -> Result<()> {
    let app = MainWindow::new()?;

    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;

    PROJ.set(proj)
        .map_err(|_| anyhow!("Failed to set project dirs"))?;

    config::init()?;
    titles::init()?;
    tasks::init(app.as_weak())?;

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    refresh_dir_name(&app);
    refresh_games(&app)?;
    refresh_hbc_apps(&app)?;
    refresh_disk_usage(&app);

    watcher::init(&app)?;

    let weak = app.as_weak();
    app.on_choose_mount_point(move || {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            if let Err(e) = config::update(|config| {
                config.mount_point.clone_from(&dir);
            }) {
                show_err(e);
            }

            if let Some(handle) = weak.upgrade() {
                refresh_dir_name(&handle);
                if let Err(e) = refresh_games(&handle) {
                    show_err(e);
                }
                if let Err(e) = refresh_hbc_apps(&handle) {
                    show_err(e);
                }
                refresh_disk_usage(&handle);
                if let Err(e) = watcher::init(&handle) {
                    show_err(e);
                }
            } else {
                show_err(anyhow!("Failed to upgrade weak reference"));
            }
        }
    });

    app.on_open_url(|url| {
        if let Err(e) = open::that(url) {
            show_err(e);
        }
    });

    app.on_add_games(|| {
        if let Err(e) = add_games::add_games() {
            show_err(e);
        }
    });

    app.on_wii_output_format_changed(|format| {
        if let Err(e) = config::update(|config| {
            config.wii_output_format = format;
        }) {
            show_err(e);
        }
    });

    app.on_archive_format_changed(|format| {
        if let Err(e) = config::update(|config| {
            config.archive_format = format;
        }) {
            show_err(e);
        }
    });

    app.on_remove_update_partition_changed(|enabled| {
        if let Err(e) = config::update(|config| {
            config.scrub_update_partition = enabled;
        }) {
            show_err(e);
        }
    });

    app.on_remove_game(|path| {
        if MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!("Are you sure you want to remove {path} ?"))
            .set_level(MessageLevel::Warning)
            .set_buttons(MessageButtons::YesNo)
            .show()
            == MessageDialogResult::Yes
            && let Err(e) = std::fs::remove_dir_all(path) {
                show_err(e);
            }
    });

    app.on_get_tasks_count(tasks::count);

    if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
        updater::check();
    }

    app.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run().inspect_err(|e| show_err(e))
}
