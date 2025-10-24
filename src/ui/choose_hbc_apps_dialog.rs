// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps};
use eframe::egui;
use std::path::PathBuf;

pub fn update(ctx: &egui::Context, app: &mut App) {
    if app.choose_hbc_apps.show(ctx).selected() && !app.choose_hbc_apps.selection().is_empty() {
        hbc_apps::spawn_install_apps_task(
            app,
            app.choose_hbc_apps
                .selection()
                .iter()
                .map(PathBuf::from)
                .collect(),
        );
    }
}
