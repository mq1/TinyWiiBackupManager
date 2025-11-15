// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    ui::UiAction,
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app_state: &AppState, ui_buffers: &mut UiBuffers, game_i: u16) {
    let game = &app_state.games[game_i as usize];
    let text = format!("Are you sure you want to delete {}?", &game.display_title);

    egui::Modal::new("delete_game".into()).show(ctx, |ui| {
        ui.heading("⚠ Delete Game");

        ui.add_space(10.);

        ui.label(text);

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Delete").clicked() {
                ui_buffers.action = Some(UiAction::DeleteGame(game_i));
            }

            if ui.button("❌ Cancel").clicked() {
                ui_buffers.action = Some(UiAction::CloseModal);
            }
        });
    });
}
