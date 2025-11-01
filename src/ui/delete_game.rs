// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;
use std::fs;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("delete_game".into());
    let mut close = false;
    let mut refresh = false;

    if let Some(game) = &app.deleting_game {
        let text = format!("Are you sure you want to delete {}?", &game.display_title);

        modal.show(ctx, |ui| {
            ui.heading("⚠ Delete Game");

            ui.add_space(10.);

            ui.label(text);

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Delete").clicked() {
                    if let Err(e) = fs::remove_dir_all(&game.path) {
                        app.toasts.error(e.to_string());
                    }

                    close = true;
                    refresh = true;
                }

                if ui.button("❌ Cancel").clicked() {
                    close = true;
                }
            });
        });
    }

    if close {
        app.deleting_game = None;
    }

    if refresh {
        app.refresh_games(ctx);
    }
}
