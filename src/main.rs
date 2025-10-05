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
pub mod wiitdb;

use crate::{fs::get_disk_usage, tasks::TASK_PROCESSOR};
use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel};
use std::{
    fmt::Display,
    rc::Rc,
    sync::{Mutex, OnceLock},
};

slint::include_modules!();

pub static PROJ: OnceLock<ProjectDirs> = OnceLock::new();
static WATCHER: Mutex<Option<RecommendedWatcher>> = Mutex::new(None);

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

fn watch(handle: &MainWindow) -> Result<()> {
    let mount_point = config::get().mount_point;
    if mount_point.as_os_str().is_empty() {
        return Ok(());
    }

    let weak = handle.as_weak();
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(notify::Event {
            kind:
                notify::EventKind::Modify(_)
                | notify::EventKind::Create(_)
                | notify::EventKind::Remove(_),
            ..
        }) = res
        {
            weak.upgrade_in_event_loop(|handle| {
                refresh_games(&handle).err().map(show_err);
                refresh_hbc_apps(&handle).err().map(show_err);
                refresh_disk_usage(&handle);
            })
            .err()
            .map(show_err);
        }
    })?;

    watcher.watch(&mount_point.join("wbfs"), RecursiveMode::NonRecursive)?;
    watcher.watch(&mount_point.join("games"), RecursiveMode::NonRecursive)?;
    watcher.watch(&mount_point.join("apps"), RecursiveMode::NonRecursive)?;

    WATCHER
        .lock()
        .map_err(|_| anyhow!("Mutex poisoned"))?
        .replace(watcher);

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

    // Initialize task processor/queue
    TASK_PROCESSOR
        .set(tasks::TaskProcessor::new(app.as_weak()))
        .map_err(|_| anyhow!("Failed to initialize task processor"))?;

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    refresh_dir_name(&app);
    refresh_games(&app)?;
    refresh_hbc_apps(&app)?;
    refresh_disk_usage(&app);

    watch(&app)?;

    let weak = app.as_weak();
    app.on_choose_mount_point(move || {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            config::update(|config| {
                config.mount_point.clone_from(&dir);
            })
            .err()
            .map(show_err);

            if let Some(handle) = weak.upgrade() {
                refresh_dir_name(&handle);
                refresh_games(&handle).err().map(show_err);
                refresh_hbc_apps(&handle).err().map(show_err);
                refresh_disk_usage(&handle);
                watch(&handle).err().map(show_err);
            } else {
                show_err(&anyhow!("Failed to upgrade weak reference"));
            }
        }
    });

    app.on_open_game_info(move |id| {
        open::that(format!("https://www.gametdb.com/Wii/{id}"))
            .err()
            .map(show_err);
    });

    app.on_open_game_dir(move |path| {
        open::that(path).err().map(show_err);
    });

    app.on_add_games(move || {
        add_games::add_games().err().map(show_err);
    });

    app.on_wii_output_format_changed(|format| {
        config::update(|config| {
            config.wii_output_format = format;
        })
        .err()
        .map(show_err);
    });

    app.on_archive_format_changed(|format| {
        config::update(|config| {
            config.archive_format = format;
        })
        .err()
        .map(show_err);
    });

    app.on_remove_update_partition_changed(|enabled| {
        config::update(|config| {
            config.scrub_update_partition = enabled;
        })
        .err()
        .map(show_err);
    });

    app.on_remove_game(|path| {
        if MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!("Are you sure you want to remove {path} ?"))
            .set_level(MessageLevel::Warning)
            .set_buttons(MessageButtons::YesNo)
            .show()
            == MessageDialogResult::Yes
        {
            std::fs::remove_dir_all(path).err().map(show_err);
        }
    });

    app.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run().inspect_err(|e| show_err(e))
}
