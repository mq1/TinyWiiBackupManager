// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod convert;
mod covers;
mod dialogs;
mod disc_info;
mod drive_info;
mod games;
mod homebrew_apps;
mod notification;
mod osc;
mod state;
mod update;
mod util;

#[cfg(windows)]
mod window_color;

use crate::{state::new_state, update::update};
use anyhow::{Result, bail};
use slint::{ComponentHandle, ModelRc, SharedString, ToSharedString};
use smallvec::SmallVec;
use std::process::Command;
use twbm_core::data_dir::DATA_DIR;

slint::include_modules!();

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    std::process::exit(0);
}

fn main() -> Result<()> {
    if DATA_DIR.as_os_str().is_empty() {
        bail!("Failed to get data dir");
    }

    let app = AppWindow::new()?;
    let mut state = new_state();

    let logic = app.global::<Logic<'_>>();
    logic.set_app_version(env!("CARGO_PKG_VERSION").to_shared_string());
    logic.set_data_dir(DATA_DIR.to_string_lossy().to_shared_string());
    logic.set_config(DisplayedConfig::from(&state.config));
    logic.set_games(ModelRc::from(state.filtered_games.clone()));
    logic.set_homebrew_apps(ModelRc::from(state.filtered_homebrew_apps.clone()));
    logic.set_osc_apps(ModelRc::from(state.filtered_osc_apps.clone()));
    logic.set_notifications(ModelRc::from(state.notifications.clone()));
    logic.set_conversion_queue(ModelRc::from(state.conversion_queue.clone()));
    logic.set_conversion_queue_buffer(ModelRc::from(state.conversion_queue_buffer.clone()));

    let mut action_queue = SmallVec::new();
    let weak = logic.as_weak();
    let window_handle = app.window().window_handle();
    logic.on_dispatch(move |action, args| {
        action_queue.push((action, args));

        while let Some((action, args)) = action_queue.pop() {
            update(
                &mut state,
                &weak,
                &window_handle,
                action,
                args,
                &mut action_queue,
            );
        }
    });

    logic.invoke_dispatch(Action::RefreshAll, SharedString::new());

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
