// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, known_mount_points};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_mount_point.update(ctx);

    if let Some(path) = app.choose_mount_point.take_picked() {
        let is_known = known_mount_points::check(&app.data_dir, &path).unwrap_or(true);

        if !is_known {
            app.notifications.show_info_long("New Drive detected, a path normalization run is recommended\nYou can find it in the 🔧 Tools page");
        }

        app.config.contents.mount_point = path;
        app.refresh_games(ctx);
        app.refresh_hbc_apps(ctx);

        if let Err(e) = app.config.write() {
            app.notifications.show_err(e);
        }
    }
}
