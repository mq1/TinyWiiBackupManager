// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::error_handling::show_anyhow_error;
use anyhow::anyhow;
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

            // dot_clean button
            #[cfg(target_os = "macos")]
            {
                let btn = ui
                    .button("👻 Clean MacOS ._ files")
                    .on_hover_text("Run dot_clean in the wbfs parent directory");

                if btn.clicked() {
                    if let Err(e) = app.run_dot_clean() {
                        show_anyhow_error("Error running dot_clean", &e);
                    }
                }
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ Wiki", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");

                ui.label("•");

                ui.label(format!(
                    "size: {:.2} GiB",
                    app.wbfs_dir_size as f64 / 1024.0 / 1024.0 / 1024.0
                ));
                if ui
                    .hyperlink("WBFS")
                    .on_hover_text("Open the WBFS directory")
                    .clicked()
                {
                    if let Err(e) = open::that(&app.wbfs_dir) {
                        show_anyhow_error("Error", &anyhow!(e));
                    }
                }

                ui.label("•");

                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}