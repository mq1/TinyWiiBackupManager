// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, disc_info::DiscInfo};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_games.update(ctx);

    if let Some(mut paths) = app.choose_games.take_picked_multiple() {
        // We'll get those later with get_overflow_file
        paths.retain(|path| !path.ends_with(".part1.iso"));

        app.choosing_games = paths
            .into_iter()
            .map(DiscInfo::from_main_file)
            .filter_map(Result::ok)
            .filter(|info| {
                app.wii_games
                    .iter()
                    .chain(app.gc_games.iter())
                    .all(|game| game.id != info.header.game_id)
            })
            .collect();

        if app.choosing_games.is_empty() {
            app.toasts.info("No new games were selected");
        }
    }
}
