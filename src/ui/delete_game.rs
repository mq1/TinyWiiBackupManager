// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;
use std::fs;

pub fn update(ctx: &egui::Context, app: &mut App, game_i: u16) {
    let modal = egui::Modal::new("delete_game".into());
    let mut action = Action::None;

    let game = &app.games[game_i as usize];
    let text = format!("Are you sure you want to delete {}?", &game.display_title);

    modal.show(ctx, |ui| {
        ui.heading("⚠ Delete Game");

        ui.add_space(10.);

        ui.label(text);

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Delete").clicked() {
                action = Action::Delete;
            }

            if ui.button("❌ Cancel").clicked() {
                action = Action::Cancel;
            }
        });
    });

    match action {
        Action::None => {}
        Action::Delete => {
            if let Err(e) = fs::remove_dir_all(&game.path) {
                app.notifications.show_err(e.into());
            }

            app.current_modal = ui::Modal::None;
            app.refresh_games(ctx);
        }
        Action::Cancel => {
            app.current_modal = ui::Modal::None;
        }
    }
}

enum Action {
    None,
    Delete,
    Cancel,
}
