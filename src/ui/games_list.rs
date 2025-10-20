// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, disc_info::DiscInfo};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::remainder())
        .columns(Column::auto(), 3)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("🏷 Title");
            });
            header.col(|ui| {
                ui.heading("🎮 Console   ");
            });
            header.col(|ui| {
                ui.heading("⚖ Size   ");
            });
            header.col(|ui| {
                ui.heading("☞ Actions");
            });
        })
        .body(|mut body| {
            for game in app.filtered_games.lock().iter() {
                body.row(20., |mut row| {
                    row.col(|ui| {
                        ui.label(&game.display_title);
                    });
                    row.col(|ui| {
                        ui.label(if game.is_wii { "🎾 Wii" } else { "🎲 GC" });
                    });
                    row.col(|ui| {
                        ui.label(game.size.to_string());
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui.button("ℹ").on_hover_text("Show Game Info").clicked() {
                                app.disc_info =
                                    Some(DiscInfo::from_game_dir(&game.path).unwrap_or_default());
                            }
                            // Archive button
                            if ui
                                .button("📦")
                                .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                                .clicked()
                            {}

                            // Integrity check button
                            if ui.button("☑").on_hover_text("Integrity Check").clicked() {}

                            // Remove button
                            if ui.button("🗑").on_hover_text("Remove Game").clicked() {
                                app.removing_game = Some(game.clone());
                            }
                        });
                    });
                });
            }
        });
}
