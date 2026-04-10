// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod checksum;
mod config;
mod conv_queue;
mod convert;
mod covers;
mod data_dir;
mod dialogs;
mod disc_info;
mod drive_info;
mod extensions;
mod game;
mod game_list;
mod hbc_app;
mod hbc_app_list;
mod id_map;
mod notification;
mod results;
mod state;
mod util;

#[cfg(windows)]
mod window_color;

#[cfg(windows)]
mod xp_dialogs;

use crate::data_dir::get_data_dir;
use anyhow::{Result, bail};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::{fs, path::Path, process::Command, rc::Rc};

slint::include_modules!();

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let data_dir = Box::leak(Box::new(get_data_dir()?));
    let app = AppWindow::new()?;

    app.global::<State<'_>>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    app.global::<State<'_>>()
        .set_data_dir(data_dir.to_string_lossy().to_shared_string());

    app.global::<Rust<'_>>()
        .on_load_config(|| Config::load(data_dir));

    app.global::<Rust<'_>>()
        .on_open(|uri| open::that(&uri).into());

    let weak = app.as_weak();
    app.global::<Rust<'_>>().on_pick_mount_point(move || {
        let app = weak.upgrade().unwrap();

        match dialogs::pick_mount_point(app.window()) {
            Some(path) => path.to_string_lossy().to_shared_string(),
            None => SharedString::new(),
        }
    });

    app.global::<Rust<'_>>()
        .on_write_config(|config| config.write().into());

    app.global::<Rust<'_>>()
        .on_get_drive_info(|path| DriveInfo::from_path(&path));

    app.global::<Rust<'_>>()
        .on_delete_dir(|path| fs::remove_dir_all(path).into());

    app.global::<Rust<'_>>()
        .on_get_game_list(|path, sort_by| GameList::new(Path::new(&path), data_dir, sort_by));

    app.global::<Rust<'_>>()
        .on_get_hbc_app_list(|path, sort_by| HbcAppList::new(Path::new(&path), sort_by));

    app.global::<Rust<'_>>()
        .on_filter_games(|games, query| game_list::fuzzy_search(&games, &query));

    app.global::<Rust<'_>>()
        .on_filter_hbc_apps(|apps, query| hbc_app_list::fuzzy_search(&apps, &query));

    app.global::<Rust<'_>>()
        .on_get_disc_info(|game_dir| DiscInfo::try_from_game_dir(Path::new(&game_dir)).into());

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_pick_games(move |existing_games, conf, drive_info| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue = QueuedConversion::make_queue(paths, &existing_ids, &conf, &drive_info);
            let model = VecModel::from(queue);
            ModelRc::from(Rc::new(model))
        });

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_pick_games_r(move |existing_games, conf, drive_info| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games_r(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue = QueuedConversion::make_queue(paths, &existing_ids, &conf, &drive_info);
            let model = VecModel::from(queue);
            ModelRc::from(Rc::new(model))
        });

    app.global::<Rust<'_>>().on_sort_games(|games, sort_by| {
        let compare = game::get_compare_fn(sort_by);
        let model = games.sort_by(compare);
        ModelRc::from(Rc::new(model))
    });

    app.global::<Rust<'_>>()
        .on_add_notification(|notifications, notification| {
            notifications
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .push(notification);
        });

    app.global::<Rust<'_>>()
        .on_close_notification(|notifications, i| {
            #[allow(clippy::cast_sign_loss)]
            notifications
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .remove(i as usize);
        });

    app.global::<Rust<'_>>().on_add_to_queue(|queue, new| {
        let model = queue
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        let new_model = new
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        while new.row_count() > 0 {
            model.push(new_model.remove(new.row_count() - 1));
        }
    });

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_archive_game(move |queue, game| {
            let app = weak.unwrap();
            let mut result = EmptyResult::default();

            let Some(out_path) = dialogs::save_game(app.window(), &game) else {
                result.err.push_str("No output path selected");
                return result;
            };

            let conv = match QueuedConversion::new_archive(&game, &out_path) {
                Ok(conv) => conv,
                Err(e) => {
                    result.err = e.to_shared_string();
                    return result;
                }
            };

            let model = queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            model.push(conv);
            result
        });

    #[cfg(windows)]
    {
        let weak = app.as_weak();
        app.global::<Rust<'_>>()
            .on_set_window_color(move |is_dark| {
                let app = weak.upgrade().unwrap();
                window_color::set(app.window(), is_dark);
            });
    }

    app.global::<State<'_>>().handle_callbacks();

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
