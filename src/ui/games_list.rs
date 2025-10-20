// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto().resizable(true))
        .column(Column::remainder())
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("🏷 Game");
            });
            header.col(|ui| {
                ui.heading("🎮 Console");
            });
        })
        .body(|mut body| {
            for game in app.filtered_games.lock().iter() {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.label(&game.display_title);
                    });
                    row.col(|ui| {
                        ui.label(if game.is_wii { "🎾 Wii" } else { "🎲 GC" });
                    });
                });
            }
        });
}
