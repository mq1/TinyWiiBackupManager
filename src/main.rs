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
pub mod install;
pub mod oscwii;
pub mod tasks;
pub mod titles;
pub mod updater;
pub mod util;
pub mod watcher;
pub mod wiiload;
pub mod wiitdb;

use crate::{tasks::TaskProcessor, titles::Titles, watcher::init_watcher};
use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use notify::RecommendedWatcher;
use path_slash::PathBufExt;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use slint::{ModelRc, ToSharedString, VecModel, Weak};
use std::{
    fmt::Display,
    fs,
    path::Path,
    rc::Rc,
    sync::{Arc, Mutex},
};

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
    let games_model = ModelRc::from(Rc::new(VecModel::from(games)));
    handle.set_games(games_model.clone());
    handle.set_filtered_games(games_model); // Also update the filtered list
    Ok(())
}

fn refresh_hbc_apps(handle: &MainWindow, mount_point: &Path) -> Result<()> {
    let hbc_apps = hbc_apps::list(mount_point)?;
    let hbc_apps_model = ModelRc::from(Rc::new(VecModel::from(hbc_apps)));
    handle.set_hbc_apps(hbc_apps_model.clone());
    handle.set_filtered_hbc_apps(hbc_apps_model); // Also update the filtered list
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
    config.mount_point = dir.to_slash_lossy().to_shared_string();
    config.save(data_dir)?;

    refresh_dir_name(&handle, &dir);
    refresh_games(&handle, &dir, titles)?;
    refresh_hbc_apps(&handle, &dir)?;
    refresh_disk_usage(&handle, &dir);

    let new_watcher = init_watcher(weak.clone(), &dir, titles)?;
    let mut guard = watcher.lock().map_err(|_| anyhow!("Mutex poisoned"))?;
    *guard = new_watcher;

    Ok(())
}

fn run() -> Result<()> {
    let proj = ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;
    let data_dir = proj.data_dir().to_path_buf();
    fs::create_dir_all(&data_dir)?;

    let app = MainWindow::new()?;
    let config = Config::load(&data_dir);
    let mount_point = Path::new(&config.mount_point);
    let titles = Arc::new(Titles::load(&data_dir)?);
    let task_processor = Arc::new(TaskProcessor::init(app.as_weak())?);

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    let watcher = Arc::new(Mutex::new(init_watcher(
        app.as_weak(),
        mount_point,
        &titles,
    )?));

    refresh_dir_name(&app, mount_point);
    refresh_games(&app, mount_point, &titles)?;
    refresh_hbc_apps(&app, mount_point)?;
    refresh_disk_usage(&app, mount_point);

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
    app.on_update_config(move |new_config| {
        if let Err(e) = new_config.save(&data_dir_clone) {
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

    let data_dir_clone = data_dir.clone();
    app.on_open_data_dir(move || {
        if let Err(e) = open::that(&data_dir_clone) {
            show_err(e);
        }
    });

    let data_dir_clone = data_dir.clone();
    let task_processor_clone = task_processor.clone();
    app.on_add_apps(move || {
        if let Err(e) = install::install_apps(&data_dir_clone, &task_processor_clone) {
            show_err(e);
        }
    });

    app.on_push_zip(|wii_ip| {
        if let Some(path) = FileDialog::new()
            .set_title("Select Wii HBC App")
            .add_filter("Wii App", &["zip", "ZIP"])
            .pick_file()
        {
            if let Err(e) = wiiload::push(&path, &wii_ip) {
                show_err(e);
            }
        }
    });

    let data_dir_clone = data_dir.clone();
    let weak = app.as_weak();
    app.on_update_oscwii_apps(move || {
        let res = oscwii::Apps::load(&data_dir_clone);

        if let Err(e) = res {
            show_err(e);
        } else if let Ok(apps) = res {
            let model = apps.get_model();

            if let Some(handle) = weak.upgrade() {
                handle.set_oscwii_apps(model);
            } else {
                show_err(anyhow!("Failed to upgrade main window"));
            }
        }
    });

    let weak = app.as_weak();
    app.on_update_filtered_oscwii_apps(move |filter| {
        if let Some(handle) = weak.upgrade() {
            let apps = handle.get_oscwii_apps();
            if filter.is_empty() {
                handle.set_filtered_oscwii_apps(apps);
                return;
            }

            let filter = filter.to_lowercase();
            let filtered = apps.filter(move |app| app.name.to_lowercase().contains(&*filter));
            handle.set_filtered_oscwii_apps(ModelRc::from(Rc::new(filtered)));
        } else {
            show_err(anyhow!("Failed to upgrade main window"));
        }
    });

    let weak = app.as_weak();
    app.on_update_filtered_hbc_apps(move |filter| {
        if let Some(handle) = weak.upgrade() {
            let apps = handle.get_hbc_apps();
            if filter.is_empty() {
                handle.set_filtered_hbc_apps(apps);
                return;
            }

            let filter = filter.to_lowercase();
            let filtered = apps.filter(move |app| app.name.to_lowercase().contains(&*filter));
            handle.set_filtered_hbc_apps(ModelRc::from(Rc::new(filtered)));
        } else {
            show_err(anyhow!("Failed to upgrade main window"));
        }
    });

    let weak = app.as_weak();
    app.on_update_filtered_games(move |filter| {
        if let Some(handle) = weak.upgrade() {
            let games = handle.get_games();
            if filter.is_empty() {
                handle.set_filtered_games(games);
                return;
            }

            let filter = filter.to_lowercase();
            let filtered = games.filter(move |game| game.display_title.to_lowercase().contains(&*filter));
            handle.set_filtered_games(ModelRc::from(Rc::new(filtered)));
        } else {
            show_err(anyhow!("Failed to upgrade main window"));
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
