// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;

/// Renders the top menu bar
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            let is_converting = app.conversion_promise.is_some();

            // Add games button
            ui.add_enabled_ui(!is_converting, |ui| {
                if ui
                    .button("➕ Add Game(s)")
                    .on_hover_text("Add a new game to the WBFS directory")
                    .clicked()
                {
                    app.add_isos();
                }
            });

            // Game counter
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}
