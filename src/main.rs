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
mod tasks;
pub mod titles;
pub mod updater;
pub mod util;
pub mod verify;
pub mod wiiload;
pub mod wiitdb;

use crate::{tasks::TaskProcessor, titles::Titles};
use anyhow::{Result, anyhow};
use parking_lot::Mutex;
use path_slash::PathBufExt;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use slint::{ModelRc, SharedString, ToSharedString, VecModel, Weak};
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

slint::include_modules!();

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

fn refresh_games(
    handle: &MainWindow,
    mount_point: &Path,
    titles: Arc<Mutex<Titles>>,
) -> Result<()> {
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
    titles: Arc<Mutex<Titles>>,
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
    let titles = Arc::new(Mutex::new(Titles::empty()));
    let task_processor = Arc::new(TaskProcessor::init(app.as_weak()));
    let lazy_task_processor = Arc::new(TaskProcessor::init(app.as_weak()));

    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    refresh_dir_name(&app, mount_point);
    refresh_games(&app, mount_point, titles.clone())?;
    refresh_hbc_apps(&app, mount_point)?;
    refresh_disk_usage(&app, mount_point);
    app.set_config(config);

    let titles_clone = titles.clone();
    let data_dir_clone = data_dir.clone();
    let task_processor_clone = task_processor.clone();
    app.on_choose_mount_point(move || {
        let titles_clone = titles_clone.clone();
        let data_dir_clone = data_dir_clone.clone();

        task_processor_clone.run_now(Box::new(move |weak| {
            choose_mount_point(weak, titles_clone, &data_dir_clone)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_open_url(move |url| {
        task_processor_clone.run_now(Box::new(move |_| {
            open::that(url)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_add_games(move |config| {
        task_processor_clone.spawn(Box::new(move |weak| {
            convert::add_games(&config, weak)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    let data_dir_clone = data_dir.clone();
    app.on_update_config(move |new_config| {
        let data_dir_clone = data_dir_clone.clone();

        task_processor_clone.run_now(Box::new(move |_| {
            new_config.save(&data_dir_clone)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_remove_dir(move |path| {
        task_processor_clone.run_now(Box::new(move |_| {
            fs::remove_dir_all(&path)?;
            Ok(String::new())
        }));
    });

    let data_dir_clone = data_dir.clone();
    let task_processor_clone = task_processor.clone();
    app.on_open_data_dir(move || {
        let data_dir_clone = data_dir_clone.clone();

        task_processor_clone.run_now(Box::new(move |_| {
            open::that(&data_dir_clone)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_add_apps(move |config| {
        task_processor_clone.spawn(Box::new(move |weak| {
            hbc_apps::add_apps(&config, weak)?;
            Ok(String::new())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_push_file(move |wii_ip| {
        task_processor_clone.spawn(Box::new(move |weak| {
            let excluded_files = wiiload::push_file(&wii_ip, weak)?;
            let mut msg = "Push successful.".to_string();
            if !excluded_files.is_empty() {
                msg.push_str(&format!("\nExcluded files: {}", excluded_files.join(", ")));
            }
            Ok(msg)
        }));
    });

    app.on_get_filtered_oscwii_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.name_lower.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_get_filtered_hbc_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.name_lower.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_get_filtered_games(move |games, filter, show_wii, show_gc| {
        if !show_wii && !show_gc {
            return ModelRc::from(Rc::new(VecModel::from_slice(&[])));
        }

        let filtered = games.filter(move |game| {
            (show_wii && game.console == Console::Wii)
                || (show_gc && game.console == Console::GameCube)
        });

        if filter.is_empty() {
            return ModelRc::from(Rc::new(filtered));
        }

        let filtered = filtered.filter(move |game| game.display_title_lower.contains(&*filter));

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

    let task_processor_clone = task_processor.clone();
    app.on_dot_clean(move |mount_point| {
        task_processor_clone.run_now(Box::new(move |_| {
            util::run_dot_clean(&mount_point)?;
            Ok("dot_clean completed successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_oscwii(move |mount_point, zip_url| {
        task_processor_clone.spawn(Box::new(move |weak| {
            hbc_apps::add_app_from_url(&mount_point, &zip_url, weak)?;
            Ok("App downloaded successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_push_oscwii(move |zip_url, wii_ip| {
        task_processor_clone.spawn(Box::new(move |weak| {
            let excluded_files = wiiload::push_oscwii(&zip_url, &wii_ip, weak)?;
            let mut msg = "Push successful.".to_string();
            if !excluded_files.is_empty() {
                msg.push_str(&format!("\nExcluded files: {}", excluded_files.join(", ")));
            }
            Ok(msg)
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_archive_game(move |game_dir, config| {
        task_processor_clone.spawn(Box::new(move |weak| {
            archive::archive_game(&game_dir, &config, weak)?;
            Ok("Archiving completed successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_wiitdb(move |mount_point| {
        task_processor_clone.spawn(Box::new(move |weak| {
            wiitdb::download(&mount_point, weak)?;
            Ok("wiitdb.xml downloaded successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_verify_game(move |game_dir| {
        task_processor_clone.spawn(Box::new(move |weak| {
            verify::verify_game(&game_dir, weak)?;
            Ok("Verification completed successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_download_all_covers(move |mount_point| {
        task_processor_clone.spawn(Box::new(move |weak| {
            covers::download_all_covers(&mount_point, weak)?;
            Ok("Covers downloaded successfully".to_string())
        }));
    });

    app.on_get_disc_info(move |mount_point| {
        disc_info::get_disc_info(&mount_point).unwrap_or_default()
    });

    let titles_clone = titles.clone();
    let task_processor_clone = task_processor.clone();
    app.on_refresh(move |mount_point| {
        let titles_clone = titles_clone.clone();

        task_processor_clone.run_now(Box::new(move |weak| {
            let mount_point = Path::new(&mount_point);

            let handle = weak
                .upgrade()
                .ok_or(anyhow!("Could not upgrade weak handle"))?;

            refresh_games(&handle, mount_point, titles_clone)?;
            refresh_hbc_apps(&handle, mount_point)?;
            refresh_disk_usage(&handle, mount_point);
            handle.invoke_apply_sorting();

            Ok(String::new())
        }));
    });

    let data_dir_clone = data_dir.clone();
    lazy_task_processor.spawn(Box::new(move |weak| {
        titles::load_titles(&data_dir_clone, weak, titles)?;
        Ok(String::new())
    }));

    if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
        lazy_task_processor.spawn(Box::new(move |weak| {
            updater::check(weak)?;
            Ok(String::new())
        }));
    }

    lazy_task_processor.spawn(Box::new(move |weak| {
        oscwii::load_oscwii_apps(&data_dir, weak)?;
        Ok(String::new())
    }));

    app.invoke_apply_sorting();
    app.run()?;
    Ok(())
}

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
    if let Err(e) = run() {
        let _ = MessageDialog::new()
            .set_level(MessageLevel::Error)
            .set_title("Error")
            .set_description(e.to_string())
            .show();

        return Err(e);
    }

    Ok(())
}
