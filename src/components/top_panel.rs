// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            let add_games_button = ui
                .button("➕ Add Game(s)")
                .on_hover_text("Add a new game to the WBFS directory");

            if add_games_button.clicked() {
                app.add_isos();
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");
            });
        });
    });
}
