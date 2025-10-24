// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, convert};
use eframe::egui;
use std::path::PathBuf;

pub fn update(ctx: &egui::Context, app: &mut App) {
    if app.choose_games.show(ctx).selected() && !app.choose_games.selection().is_empty() {
        convert::spawn_add_games_task(
            app,
            app.choose_games
                .selection()
                .iter()
                .map(PathBuf::from)
                .collect(),
        );
    }
}
