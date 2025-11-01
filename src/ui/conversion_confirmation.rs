// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, convert};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("conversion_confirmation".into());

    if !app.choosing_games.is_empty() {
        modal.show(ctx, |ui: &mut egui::Ui| {
            ui.heading(format!(
                "🎮 {} Games selected for conversion",
                app.choosing_games.len()
            ));
            ui.label("(Existing games are automatically ignored)");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.)
                .show(ui, |ui| {
                    for info in &app.choosing_games {
                        ui.label(format!(
                            "⏵ {} [{}]",
                            info.header.game_title_str(),
                            info.header.game_id_str()
                        ));
                    }
                });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("✅ Start conversion").clicked() {
                    let mut games = Vec::with_capacity(app.choosing_games.len());
                    games.append(&mut app.choosing_games);

                    convert::spawn_add_games_task(app, games);
                }

                if ui.button("❌ Cancel").clicked() {
                    app.choosing_games.clear();
                }
            })
        });
    }
}
