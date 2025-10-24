// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    if app.choose_mount_point.show(ctx).selected()
        && let Some(path) = app.choose_mount_point.path()
    {
        app.config.contents.mount_point = path.to_path_buf();
        app.refresh_games(ctx);
        app.refresh_hbc_apps(ctx);

        if let Err(e) = app.config.write() {
            app.toasts.error(e.to_string());
        }
    }
}
