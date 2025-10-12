// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Don't show windows terminal
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod archive;
pub mod config;
pub mod convert;
pub mod covers;
pub mod disc_info;
pub mod extensions;
pub mod games;
pub mod hbc_apps;
pub mod http;
pub mod oscwii;
pub mod overflow_reader;
pub mod tasks;
pub mod titles;
pub mod updater;
pub mod util;
pub mod verify;
pub mod watcher;
pub mod wiiload;
pub mod wiitdb;

use crate::{tasks::TaskProcessor, titles::Titles, watcher::init_watcher};
use anyhow::{Result, anyhow};
use notify::RecommendedWatcher;
use path_slash::PathBufExt;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use slint::{ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
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
    Ok(())
}

fn refresh_hbc_apps(handle: &MainWindow, mount_point: &Path) -> Result<()> {
    let hbc_apps = hbc_apps::list(mount_point)?;
    let hbc_apps_model = ModelRc::from(Rc::new(VecModel::from(hbc_apps)));
    handle.set_hbc_apps(hbc_apps_model.clone());
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
    config.mount_point = dir
        .to_slash()
        .ok_or(anyhow!("Invalid path"))?
        .to_shared_string();
    config.save(data_dir)?;
    handle.set_config(config);

    refresh_dir_name(&handle, &dir);
    refresh_games(&handle, &dir, titles)?;
    refresh_hbc_apps(&handle, &dir)?;
    refresh_disk_usage(&handle, &dir);
    handle.invoke_apply_sorting();
    handle.invoke_reset_filters();

    let new_watcher = init_watcher(weak.clone(), &dir, titles)?;
    let mut guard = watcher.lock().map_err(|_| anyhow!("Mutex poisoned"))?;
    *guard = new_watcher;

    Ok(())
}

fn run() -> Result<()> {
    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;

    let app = MainWindow::new()?;
    let mut config = Config::load(&data_dir);

    // If the mount point doesn't exist, erase it
    if !matches!(fs::exists(&config.mount_point), Ok(true)) {
        config.mount_point = SharedString::new();
        config.save(&data_dir)?;
    }

    // Load the mount point from the first argument
    if let Some(path) = std::env::args().nth(1) {
        config.mount_point = PathBuf::from(path)
            .to_slash()
            .ok_or(anyhow!("Invalid path"))?
            .to_shared_string();
        config.save(&data_dir)?;
    }

    let mount_point = Path::new(&config.mount_point);
    let titles = Arc::new(Titles::load(&data_dir)?);
    let task_processor = Arc::new(TaskProcessor::init(app.as_weak()));

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
    app.set_config(config);

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

    let task_processor_clone = task_processor.clone();
    app.on_add_games(move |config| {
        if let Err(e) = convert::add_games(&config, &task_processor_clone) {
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

    let task_processor_clone = task_processor.clone();
    app.on_add_apps(move |config| {
        if let Err(e) = hbc_apps::add_apps(&config, &task_processor_clone) {
            show_err(e);
        }
    });

    let task_processor_clone = task_processor.clone();
    app.on_push_file(move |wii_ip| {
        if let Err(e) = wiiload::push_file(&wii_ip, &task_processor_clone) {
            show_err(e);
        }
    });

    app.on_update_filtered_oscwii_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.name_lower.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_update_filtered_hbc_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.name_lower.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_update_filtered_games(move |games, filter| {
        if filter.is_empty() {
            return games;
        }

        let filtered = games.filter(move |game| game.display_title_lower.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_sort(|config, games, apps| match config.sort_by {
        SortBy::NameAscending => (
            ModelRc::from(Rc::new(
                games.sort_by(|a, b| a.display_title_lower.cmp(&b.display_title_lower)),
            )),
            ModelRc::from(Rc::new(
                apps.sort_by(|a, b| a.name_lower.cmp(&b.name_lower)),
            )),
        ),
        SortBy::NameDescending => (
            ModelRc::from(Rc::new(
                games.sort_by(|a, b| b.display_title_lower.cmp(&a.display_title_lower)),
            )),
            ModelRc::from(Rc::new(
                apps.sort_by(|a, b| b.name_lower.cmp(&a.name_lower)),
            )),
        ),
        SortBy::SizeAscending => (
            ModelRc::from(Rc::new(games.sort_by(|a, b| a.size_mib.cmp(&b.size_mib)))),
            ModelRc::from(Rc::new(apps.sort_by(|a, b| a.size_mib.cmp(&b.size_mib)))),
        ),
        SortBy::SizeDescending => (
            ModelRc::from(Rc::new(games.sort_by(|a, b| b.size_mib.cmp(&a.size_mib)))),
            ModelRc::from(Rc::new(apps.sort_by(|a, b| b.size_mib.cmp(&a.size_mib)))),
        ),
    });

    app.on_dot_clean(|mount_point| {
        if let Err(e) = util::run_dot_clean(&mount_point) {
            show_err(e);
        } else {
            let _ = MessageDialog::new()
                .set_title("Success")
                .set_description("dot_clean completed successfully")
                .set_level(MessageLevel::Info)
                .show();
        }
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_oscwii(move |mount_point, zip_url| {
        hbc_apps::add_app_from_url(
            PathBuf::from(&mount_point),
            zip_url.to_string(),
            &task_processor_clone,
        );
    });

    let task_processor_clone = task_processor.clone();
    app.on_push_oscwii(move |zip_url, wii_ip| {
        wiiload::push_oscwii(
            zip_url.to_string(),
            wii_ip.to_string(),
            &task_processor_clone,
        );
    });

    let task_processor_clone = task_processor.clone();
    app.on_archive_game(move |game, config| {
        if let Err(e) =
            archive::archive_game(PathBuf::from(&game.path), &config, &task_processor_clone)
        {
            show_err(e);
        }
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_wiitdb(move |mount_point| {
        wiitdb::download(PathBuf::from(&mount_point), &task_processor_clone);
    });

    let task_processor_clone = task_processor.clone();
    app.on_verify_game(move |game_dir| {
        if let Err(e) = verify::verify_game(Path::new(&game_dir), &task_processor_clone) {
            show_err(e);
        }
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_all_covers(move |mount_point| {
        covers::download_all_covers(PathBuf::from(&mount_point), &task_processor_clone);
    });

    app.on_get_disc_info(move |mount_point| {
        let res = disc_info::get_disc_info(Path::new(&mount_point));
        match res {
            Ok(info) => info,
            Err(e) => {
                show_err(e);
                DiscInfo::default()
            }
        }
    });

    if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
        updater::check(&task_processor);
    }

    oscwii::load_oscwii_apps(&data_dir, &task_processor);

    app.invoke_apply_sorting();
    app.invoke_reset_filters();
    app.run()?;
    Ok(())
}

#[cfg(feature = "portable")]
fn get_data_dir() -> Result<PathBuf> {
    // For portable builds, use a directory next to the executable
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or(anyhow!("Could not get executable directory"))?;
    let data_dir = exe_dir.join("TinyWiiBackupManager-data");
    Ok(data_dir)
}

#[cfg(not(feature = "portable"))]
fn get_data_dir() -> Result<PathBuf> {
    // For standard builds, use the system's app data directory
    let proj = directories::ProjectDirs::from("it", "mq1", env!("CARGO_PKG_NAME"))
        .ok_or(anyhow!("Failed to get project dirs"))?;
    Ok(proj.data_dir().to_path_buf())
}

fn main() -> Result<()> {
    run().inspect_err(|e| show_err(e))
}
