// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, convert};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_games.update(ctx);

    if let Some(paths) = app.choose_games.take_picked_multiple() {
        convert::spawn_add_games_task(app, paths);
    }
}
