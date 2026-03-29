// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod data_dir;
mod dialogs;
mod game;
mod game_list;
mod id_map;
mod results;
mod util;

#[cfg(target_vendor = "pc")]
mod window_color;

#[cfg(target_vendor = "win7")]
mod xp_dialogs;

use crate::{data_dir::get_data_dir, id_map::ID_MAP};
use anyhow::{Result, bail};
use slint::{SharedString, ToSharedString};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
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

    app.global::<State<'_>>().set_config(config);

    app.global::<State<'_>>()
        .set_drive_usage(util::get_drive_usage(&mount_point));

    app.global::<State<'_>>()
        .set_game_list(GameList::new(&mount_point, data_dir));

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
        .on_get_drive_usage(|path| util::get_drive_usage(Path::new(&path)));

    app.global::<Rust<'_>>()
        .on_delete_dir(|path| fs::remove_dir_all(path).into());

    app.global::<Rust<'_>>()
        .on_get_game_list(|path| GameList::new(Path::new(&path), data_dir));

    app.global::<Rust<'_>>()
        .on_filter_games(|mut game_list, query| {
            game_list.fuzzy_search(&query);
            game_list
        });

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
