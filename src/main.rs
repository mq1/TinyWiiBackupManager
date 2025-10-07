// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Don't show windows terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod config;
pub mod convert;
pub mod covers;
pub mod extensions;
pub mod games;
pub mod hbc_apps;
pub mod http;
pub mod tasks;
pub mod titles;
pub mod updater;
pub mod util;
pub mod watcher;
pub mod wiitdb;

use crate::{config::Config, tasks::TaskProcessor, titles::Titles, watcher::init_watcher};
use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use notify::RecommendedWatcher;
use parking_lot::Mutex;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel, Weak};
use std::{fmt::Display, fs, path::Path, rc::Rc, sync::Arc};

slint::include_modules!();

fn show_err(e: impl Display) {
    let _ = MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title("Error")
        .set_description(e.to_string())
        .show();
}

fn refresh_dir_name(handle: &MainWindow, mount_point: &Path) {
    let dir_name = mount_point
        .file_name()
        .unwrap_or(mount_point.as_os_str())
        .to_str()
        .unwrap_or_default();

    handle.set_dir_name(dir_name.to_shared_string());
}

fn refresh_disk_usage(handle: &MainWindow, mount_point: &Path) {
    if let Some(usage) = util::get_disk_usage(mount_point) {
        handle.set_disk_usage(usage.to_shared_string());
    }
}

fn refresh_games(handle: &MainWindow, mount_point: &Path, titles: &Arc<Titles>) -> Result<()> {
    let games = games::list(mount_point, titles)?;
    handle.set_games(ModelRc::from(Rc::new(VecModel::from(games))));
    Ok(())
}

fn refresh_hbc_apps(handle: &MainWindow, mount_point: &Path) -> Result<()> {
    let hbc_apps = hbc_apps::list(mount_point)?;
    handle.set_hbc_apps(ModelRc::from(Rc::new(VecModel::from(hbc_apps))));
    Ok(())
}

fn choose_mount_point(
    weak: &Weak<MainWindow>,
    titles: &Arc<Titles>,
    watcher: &Arc<Mutex<Option<RecommendedWatcher>>>,
    data_dir: &Path,
) -> Result<()> {
    let handle = weak.upgrade().ok_or(anyhow!("Failed to upgrade weak"))?;

    let dir = FileDialog::new()
        .pick_folder()
        .ok_or(anyhow!("No directory selected"))?;

    let mut config = Config::load(data_dir);
    config.mount_point = dir;
    config.save()?;

    refresh_dir_name(&handle, &config.mount_point);
    refresh_games(&handle, &config.mount_point, titles)?;
    refresh_hbc_apps(&handle, &config.mount_point)?;
    refresh_disk_usage(&handle, &config.mount_point);

    let new_watcher = init_watcher(weak.clone(), &config.mount_point, titles)?;
    *watcher.lock() = new_watcher;

    Ok(())
}

fn run() -> Result<()> {
    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;
    let data_dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&data_dir)?;

    let app = MainWindow::new()?;
    let config = Config::load(&data_dir);
    let titles = Arc::new(Titles::load(&data_dir)?);
    let task_processor = Arc::new(TaskProcessor::init(app.as_weak())?);

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    let watcher = Arc::new(Mutex::new(init_watcher(
        app.as_weak(),
        &config.mount_point,
        &titles,
    )?));

    refresh_dir_name(&app, &config.mount_point);
    refresh_games(&app, &config.mount_point, &titles)?;
    refresh_hbc_apps(&app, &config.mount_point)?;
    refresh_disk_usage(&app, &config.mount_point);

    let weak = app.as_weak();
    let titles_clone = titles.clone();
    let watcher_clone = watcher.clone();
    let data_dir_clone = data_dir.clone();
    app.on_choose_mount_point(move || {
        if let Err(e) = choose_mount_point(&weak, &titles_clone, &watcher_clone, &data_dir_clone) {
            show_err(e);
        }
    });

    app.on_open_url(|url| {
        if let Err(e) = open::that(url) {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    let task_processor_clone = task_processor.clone();
    app.on_add_games(move || {
        if let Err(e) = convert::add_games(&data_dir_clone, &task_processor_clone) {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    app.on_wii_output_format_changed(move |format| {
        let mut config = Config::load(&data_dir_clone);
        config.wii_output_format = format;
        if let Err(e) = config.save() {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    app.on_archive_format_changed(move |format| {
        let mut config = Config::load(&data_dir_clone);
        config.archive_format = format;
        if let Err(e) = config.save() {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    app.on_remove_update_partition_changed(move |enabled| {
        let mut config = Config::load(&data_dir_clone);
        config.scrub_update_partition = enabled;
        if let Err(e) = config.save() {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    app.on_always_split_changed(move |enabled| {
        let mut config = Config::load(&data_dir_clone);
        config.always_split = enabled;
        if let Err(e) = config.save() {
            show_err(e);
        }
    });

    app.on_remove_dir(|path| {
        if MessageDialog::new()
            .set_title("Remove Directory")
            .set_description(format!("Are you sure you want to remove {path} ?"))
            .set_level(MessageLevel::Warning)
            .set_buttons(MessageButtons::YesNo)
            .show()
            == MessageDialogResult::Yes
            && let Err(e) = fs::remove_dir_all(path)
        {
            show_err(e);
        }
    });

    app.on_open_data_dir(move || {
        if let Err(e) = open::that(&data_dir) {
            show_err(e);
        }
    });

    if std::env::var_os("TWBM_DISABLE_UPDATES").is_none()
        && let Err(e) = updater::check(&task_processor)
    {
        show_err(e);
    }

    app.run()?;
    Ok(())
}

fn main() -> Result<()> {
    run().inspect_err(|e| show_err(e))
}
