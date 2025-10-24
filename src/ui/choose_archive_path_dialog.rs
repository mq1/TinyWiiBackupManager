// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, archive};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    if app.choose_archive_path.show(ctx).selected()
        && let Some(out_path) = app.choose_games.path()
        && let Some(game_dir) = app.archiving_game.take()
    {
        archive::spawn_archive_game_task(app, game_dir, out_path.to_path_buf());
    }
}
