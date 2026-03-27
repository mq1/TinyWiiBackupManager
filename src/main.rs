// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod data_dir;
mod dialogs;
mod game;
mod game_id;
mod game_list;
mod id_map;

#[cfg(target_vendor = "pc")]
mod window_color;

#[cfg(target_vendor = "win7")]
mod xp_dialogs;

use crate::data_dir::get_data_dir;
use anyhow::{Result, bail};
use slint::{SharedString, ToSharedString};
use std::process::Command;

slint::include_modules!();

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    Ok(())
}

fn main() -> Result<()> {
    let app = AppWindow::new()?;
    let data_dir = get_data_dir()?;
    let config = Config::load(&data_dir);

    #[cfg(target_vendor = "pc")]
    let _ = window_color::set(app.window(), &config.contents.theme_preference);

    app.global::<State<'_>>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    app.global::<State<'_>>()
        .set_data_dir(data_dir.to_string_lossy().to_shared_string());

    app.global::<State<'_>>().set_config(config);

    app.global::<Rust<'_>>()
        .on_open(|uri| match open::that(&uri) {
            Ok(()) => SharedString::new(),
            Err(e) => e.to_shared_string(),
        });

    let weak = app.as_weak();
    app.global::<Rust<'_>>().on_pick_mount_point(move || {
        let app = weak.upgrade().unwrap();

        match dialogs::pick_mount_point(app.window()) {
            Some(path) => path.to_string_lossy().to_shared_string(),
            None => SharedString::new(),
        }
    });

    app.global::<Rust<'_>>()
        .on_write_config(|config| match config.write() {
            Ok(()) => SharedString::new(),
            Err(e) => e.to_shared_string(),
        });

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
