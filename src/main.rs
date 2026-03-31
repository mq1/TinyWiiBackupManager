// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod convert;
mod data_dir;
mod dialogs;
mod disc_info;
mod disc_util;
mod drive_info;
mod extensions;
mod game;
mod game_list;
mod id_map;
mod results;
mod transfer_queue;
mod util;

#[cfg(target_vendor = "pc")]
mod window_color;

#[cfg(target_vendor = "win7")]
mod xp_dialogs;

use crate::{data_dir::get_data_dir, id_map::ID_MAP};
use anyhow::{Result, bail};
use slint::{Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::LazyLock,
};

slint::include_modules!();

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    Ok(())
}

fn main() -> Result<()> {
    // Decompress idmap
    let _ = std::thread::spawn(|| LazyLock::force(&ID_MAP));

    let data_dir = Box::leak(Box::new(get_data_dir()?));

    let app = AppWindow::new()?;
    let config = Config::load(data_dir);
    let mount_point = PathBuf::from(&config.contents.mount_point);

    #[cfg(target_vendor = "pc")]
    let _ = window_color::set(app.window(), &config.contents.theme_preference);

    app.global::<State<'_>>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    app.global::<State<'_>>()
        .set_data_dir(data_dir.to_string_lossy().to_shared_string());

    app.global::<State<'_>>().set_game_list(GameList::new(
        &mount_point,
        data_dir,
        &config.contents.sort_by,
    ));

    app.global::<State<'_>>().set_config(config);

    app.global::<State<'_>>()
        .set_drive_info(DriveInfo::from_path(&mount_point));

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
        .on_get_game_list(|path, sort_by| GameList::new(Path::new(&path), data_dir, &sort_by));

    app.global::<Rust<'_>>().on_filter_games(|games, query| {
        let games = game_list::fuzzy_search(games.iter(), &query);
        let model = VecModel::from(games);
        ModelRc::from(Rc::new(model))
    });

    app.global::<Rust<'_>>().on_sort_games(|games, sort_by| {
        let games = games.iter().collect::<Vec<_>>();
        let games = game_list::sort(games, &sort_by);
        let model = VecModel::from(games);
        ModelRc::from(Rc::new(model))
    });

    app.global::<Rust<'_>>()
        .on_get_disc_info(|game_dir| DiscInfo::try_from_game_dir(Path::new(&game_dir)).into());

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_pick_games(move |game_list, conf, drive_info| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games(app.window());
            let queue = transfer_queue::make_queue(paths, &game_list, &conf, &drive_info);
            let display_string = queue
                .iter()
                .map(QueuedConversion::display_string)
                .collect::<Vec<_>>()
                .join("\n");
            let model = VecModel::from(queue);
            (ModelRc::from(Rc::new(model)), display_string.into())
        });

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_start_conversion(move |conversion_queue, to_queue| {
            let mut new_queue = Vec::new();
            new_queue.extend(conversion_queue.iter());
            new_queue.extend(to_queue.iter());

            if let Some(conv) = new_queue.pop() {
                conv.run(weak.clone());
            }

            let model = VecModel::from(new_queue);
            ModelRc::from(Rc::new(model))
        });

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
