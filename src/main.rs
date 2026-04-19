// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod archive;
mod checksum;
mod config;
mod convert;
mod covers;
mod data_dir;
mod dialogs;
mod disc_info;
mod drive_info;
mod extensions;
mod game;
mod homebrew_app;
mod homebrew_app_meta;
mod id_map;
mod model;
mod notification;
mod osc;
mod rust_callbacks;
mod scrub;
mod standard_conversion;
mod util;

use crate::{data_dir::DATA_DIR, model::AppModel};
use anyhow::{Result, bail};
use slint::ComponentHandle;
use std::process::Command;

slint::include_modules!();

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    std::process::exit(0);
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    if DATA_DIR.as_os_str().is_empty() {
        bail!("Failed to get data dir");
    }

    let config = Config::load();
    let app = AppWindow::new()?;
    let state = AppModel::new(config);

    state.init_app(&app);

    app.global::<Rust<'_>>().register_callbacks(&state, &app);

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
