// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App, game_i: u16) {
    egui::Modal::new("delete_game".into()).show(ctx, |ui| {
        ui.heading("⚠ Delete Game");

        ui.add_space(10.);

        ui.label(format!(
            "Are you sure you want to delete {}?",
            &app.games[game_i as usize].display_title
        ));

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Delete").clicked() {
                app.delete_game(ctx, game_i);
                app.close_modal();
            }

            if ui.button("❌ Cancel").clicked() {
                app.close_modal();
            }
        });
    });
}
