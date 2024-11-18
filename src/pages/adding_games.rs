// SPDX-FileCopyrightText: 2024 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::pages::Page::Games;
use eframe::egui::{self, Id};

pub fn view(ctx: &egui::Context, app: &mut App) {
    if let Some((i, total)) = *app.adding_games_progress.lock().unwrap() {
        egui::CentralPanel::default().show(ctx, |_ui| {
            egui::Area::new(Id::new("adding_games_progress"))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading("Adding games");

                    ui.add_space(10.0);
                    ui.spinner();
                    ui.add_space(10.0);

                    ui.label(&format!("{}/{}", i, total));
                });
        });
    } else {
        app.games = None;
        app.page = Games;
    }
}
