// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Don't show windows terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod fs;
pub mod games;
pub mod hbc_apps;
pub mod http;
pub mod titles;
pub mod wiitdb;

use crate::fs::get_disk_usage;
use anyhow::{Error, Result, anyhow};
use directories::ProjectDirs;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rfd::{MessageDialog, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel};
use std::{
    rc::Rc,
    sync::{Mutex, OnceLock},
};

slint::include_modules!();

pub static PROJ: OnceLock<ProjectDirs> = OnceLock::new();
static WATCHER: Mutex<Option<RecommendedWatcher>> = Mutex::new(None);

fn show_err(e: &Error) {
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

fn watch(handle: &MainWindow) {
    let mount_point = config::get().mount_point;
    if mount_point.as_os_str().is_empty() {
        return;
    }

    let weak = handle.as_weak();
    let res = notify::recommended_watcher(move |res| {
        if let Ok(notify::Event {
            kind:
                notify::EventKind::Modify(_)
                | notify::EventKind::Create(_)
                | notify::EventKind::Remove(_),
            ..
        }) = res
            && let Err(e) = weak.upgrade_in_event_loop(|handle| {
                refresh_games(&handle);
                refresh_hbc_apps(&handle);
                refresh_disk_usage(&handle);
            })
        {
            show_err(&e.into());
        }
    });

    if let Err(e) = (|| -> notify::Result<()> {
        let mut watcher = res?;
        watcher.watch(&mount_point.join("wbfs"), RecursiveMode::NonRecursive)?;
        watcher.watch(&mount_point.join("games"), RecursiveMode::NonRecursive)?;
        watcher.watch(&mount_point.join("apps"), RecursiveMode::NonRecursive)?;

        WATCHER.lock()?.replace(watcher);

        Ok(())
    })() {
        show_err(&e.into());
    }
}

fn run() -> Result<()> {
    let app = MainWindow::new()?;

    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;

    PROJ.set(proj)
        .map_err(|_| anyhow!("Failed to set project dirs"))?;

    config::init()?;
    titles::init()?;

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    refresh_dir_name(&app);
    refresh_games(&app);
    refresh_hbc_apps(&app);
    refresh_disk_usage(&app);

    watch(&app);

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
                refresh_disk_usage(&handle);
                watch(&handle);
            } else {
                show_err(&anyhow!("Failed to upgrade weak reference"));
            }
        }
    });

    app.on_open_game_info(move |id| {
        if let Err(e) = open::that(format!("https://www.gametdb.com/Wii/{id}")) {
            show_err(&e.into());
        }
    });

    app.on_open_game_dir(move |path| {
        if let Err(e) = open::that(path) {
            show_err(&e.into());
        }
    });

    app.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run().inspect_err(show_err)
}
