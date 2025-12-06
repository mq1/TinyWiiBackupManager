// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App, game_i: u16) {
    egui::Modal::new("strip_game".into()).show(ctx, |ui| {
        ui.heading(format!("{} Remove update partition", egui_phosphor::regular::WARNING));

        ui.add_space(10.);

        ui.label(format!(
            "Are you sure you want to remove the update partition from {}?\n{}This is irreversible!",
            &app.games[game_i as usize].display_title,
            egui_phosphor::regular::WARNING
        ));

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button(format!("{} Remove", egui_phosphor::regular::TRASH))
                .clicked()
            {
                app.strip_game(game_i);
                app.close_modal();
            }

            if ui
                .button(format!("{} Cancel", egui_phosphor::regular::X))
                .clicked()
            {
                app.close_modal();
            }
        });
    });
}
