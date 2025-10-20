// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_hbc_apps.update(ctx);

    if let Some(paths) = app.choose_hbc_apps.take_picked_multiple() {
        hbc_apps::spawn_install_apps_task(app, paths);
    }
}
