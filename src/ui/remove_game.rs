// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;
use std::fs;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("remove_game".into());
    let mut close = false;

    if let Some(game) = &app.removing_game {
        let text = format!("Are you sure you want to remove {}?", &game.display_title);

        modal.show(ctx, |ui| {
            ui.heading("⚠ Remove Game");

            ui.add_space(10.);

            ui.label(text);

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Remove").clicked() {
                    if let Err(e) = fs::remove_dir_all(&game.path) {
                        app.toasts.lock().error(e.to_string());
                    }
                    close = true;
                }

                if ui.button("❌ Cancel").clicked() {
                    close = true;
                }
            });
        });
    }

    if close {
        app.removing_game = None;
    }
}
