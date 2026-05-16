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

use crate::state::State;
use anyhow::{Result, bail};
use slint::{ComponentHandle, ModelRc, SharedString, ToSharedString};
use std::{collections::VecDeque, process::Command};
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
    let mut state = State::new();

    // Initialize UI state
    let ui_state = app.global::<UiState<'_>>();
    ui_state.set_app_version(env!("CARGO_PKG_VERSION").to_shared_string());
    ui_state.set_data_dir(DATA_DIR.to_string_lossy().to_shared_string());
    ui_state.set_config(DisplayedConfig::from(&state.config));
    ui_state.set_games(ModelRc::from(state.displayed_games.clone()));
    ui_state.set_homebrew_apps(ModelRc::from(state.displayed_homebrew_apps.clone()));
    ui_state.set_osc_apps(ModelRc::from(state.displayed_osc_apps.clone()));
    ui_state.set_notifications(ModelRc::from(state.notifications.clone()));
    ui_state.set_conversion_queue(ModelRc::from(state.displayed_conversion_queue.clone()));
    ui_state.set_games_to_add(ModelRc::from(state.games_to_add.clone()));

    let dispatcher = app.global::<Dispatcher<'_>>();

    // Process messages
    dispatcher.on_dispatch({
        let weak = app.as_weak();
        let mut message_queue = VecDeque::new();

        move |message, payload| {
            message_queue.push_back((message, payload));

            while let Some((message, payload)) = message_queue.pop_front() {
                state.update(&weak, message, payload, &mut message_queue);
            }
        }
    });

    // Initialize
    dispatcher.invoke_dispatch(Message::RefreshAll, SharedString::new());

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
