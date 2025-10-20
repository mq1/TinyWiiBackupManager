// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_mount_point.update(ctx);

    if let Some(path) = app.choose_mount_point.take_picked() {
        app.config.contents.mount_point = path;

        games::spawn_get_games_task(app);

        if let Err(e) = app.config.write() {
            app.toasts.lock().error(e.to_string());
        }
    }
}
