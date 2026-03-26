// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod data_dir;
mod dialogs;

#[cfg(target_vendor = "pc")]
mod window_color;

#[cfg(target_vendor = "win7")]
mod xp_dialogs;

use crate::data_dir::get_data_dir;
use anyhow::Result;
use slint::{SharedString, ToSharedString};

slint::include_modules!();

fn main() -> Result<()> {
    let app = AppWindow::new()?;
    let data_dir = get_data_dir()?;
    let config = Config::load(&data_dir);

    #[cfg(target_vendor = "pc")]
    let _ = window_color::set(app.window(), config.contents.theme_preference);

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

    app.run()?;
    Ok(())
}
