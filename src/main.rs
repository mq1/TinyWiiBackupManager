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
pub mod osc;
pub mod overflow_reader;
mod tasks;
pub mod titles;
pub mod updater;
pub mod util;
pub mod verify;
pub mod wiiload;
pub mod wiitdb;

use crate::{tasks::TaskProcessor, titles::Titles};
use anyhow::{Context, Result, anyhow};
use parking_lot::Mutex;
use rfd::{FileDialog, MessageDialog, MessageLevel};
use slint::{ModelRc, SharedString, ToSharedString, VecModel};
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

slint::include_modules!();

fn run() -> Result<()> {
    let app = MainWindow::new()?;
    app.set_app_name(env!("CARGO_PKG_NAME").to_shared_string() + " v" + env!("CARGO_PKG_VERSION"));
    app.set_is_macos(cfg!(target_os = "macos"));

    let data_dir = get_data_dir()?;
    fs::create_dir_all(&data_dir)?;

    let titles = Arc::new(Mutex::new(Titles::empty()));
    let task_processor = Arc::new(TaskProcessor::init(app.as_weak(), false));
    let lazy_task_processor = Arc::new(TaskProcessor::init(app.as_weak(), true));

    let data_dir_clone = data_dir.clone();
    let task_processor_clone = task_processor.clone();
    app.on_choose_mount_point(move |mut config| {
        let data_dir_clone = data_dir_clone.clone();

        task_processor_clone.run_now(Box::new(move |weak| {
            let dir = FileDialog::new()
                .pick_folder()
                .ok_or(anyhow!("Failed to pick folder"))?
                .canonicalize()
                .context("Failed to get canonical path")?;

            let dir_str = dir
                .to_str()
                .ok_or(anyhow!("Failed to convert path to string"))?
                .to_string();

            config.mount_point = dir_str.to_shared_string();
            config.save(&data_dir_clone)?;

            let handle = weak
                .upgrade()
                .ok_or(anyhow!("Failed to upgrade weak reference"))?;

            handle.set_config(config);
            handle.invoke_refresh_games();
            handle.invoke_refresh_hbc_apps();
            handle.invoke_refresh_disk_usage();
            handle.invoke_apply_sorting();

            Ok(String::new())
        }));
    });

    app.on_get_filename(move |path| {
        let path = Path::new(&path);

        let filename = path
            .file_name()
            .unwrap_or(path.as_os_str())
            .to_string_lossy();

        filename.to_shared_string()
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

    app.on_get_filtered_osc_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.search_str.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_get_filtered_hbc_apps(move |apps, filter| {
        if filter.is_empty() {
            return apps;
        }

        let filtered = apps.filter(move |app| app.search_str.contains(&*filter));

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

        let filtered = filtered.filter(move |game| game.search_str.contains(&*filter));

        ModelRc::from(Rc::new(filtered))
    });

    app.on_sort(|config, games, apps| match config.sort_by {
        SortBy::NameAscending => (
            ModelRc::from(Rc::new(
                games.sort_by(|a, b| a.display_title.cmp(&b.display_title)),
            )),
            ModelRc::from(Rc::new(apps.sort_by(|a, b| a.name.cmp(&b.name)))),
        ),
        SortBy::NameDescending => (
            ModelRc::from(Rc::new(
                games.sort_by(|a, b| b.display_title.cmp(&a.display_title)),
            )),
            ModelRc::from(Rc::new(apps.sort_by(|a, b| b.name.cmp(&a.name)))),
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
    app.on_download_osc(move |mount_point, zip_url| {
        task_processor_clone.spawn(Box::new(move |weak| {
            hbc_apps::add_app_from_url(&mount_point, &zip_url, weak)?;
            Ok("App downloaded successfully".to_string())
        }));
    });

    let task_processor_clone = task_processor.clone();
    app.on_push_osc(move |zip_url, wii_ip| {
        task_processor_clone.spawn(Box::new(move |weak| {
            let excluded_files = wiiload::push_osc(&zip_url, &wii_ip, weak)?;
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
    app.on_list_games(move |mount_point| {
        let mount_point = Path::new(&mount_point);
        let list = games::list(mount_point, &titles_clone).unwrap_or_default();
        ModelRc::from(Rc::new(VecModel::from(list)))
    });

    app.on_list_hbc_apps(move |mount_point| {
        let mount_point = Path::new(&mount_point);
        let list = hbc_apps::list(mount_point).unwrap_or_default();
        ModelRc::from(Rc::new(VecModel::from(list)))
    });

    app.on_get_disk_usage(move |mount_point| {
        let mount_point = Path::new(&mount_point);
        util::get_disk_usage(mount_point)
            .unwrap_or_default()
            .to_shared_string()
    });

    let mut config = Config::load(&data_dir);

    // If the mount point doesn't exist, erase it
    if !fs::exists(&config.mount_point).unwrap_or(false) {
        config.mount_point = SharedString::new();
        config.save(&data_dir)?;
    }

    // Load the mount point from the first argument
    if let Some(path) = std::env::args().nth(1)
        && let Ok(path) = PathBuf::from(path).canonicalize()
        && let Some(path_str) = path.to_str()
        && path.exists()
    {
        config.mount_point = path_str.to_shared_string();
        config.save(&data_dir)?;
    }

    app.set_config(config);

    app.invoke_refresh_hbc_apps();
    app.invoke_apply_sorting();
    app.invoke_refresh_disk_usage();

    let data_dir_clone = data_dir.clone();
    task_processor.spawn(Box::new(move |weak| {
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
        osc::load_osc_apps(&data_dir, weak)?;
        Ok(String::new())
    }));

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
