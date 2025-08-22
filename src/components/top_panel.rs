// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::error_handling::show_anyhow_error;
use anyhow::anyhow;
use eframe::egui;
use size::Size;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                // dot_clean button
                if cfg!(target_os = "macos") {
                    if ui
                        .button("👻 Clean MacOS ._ files")
                        .on_hover_text(format!("Run dot_clean in {}", app.base_dir.display()))
                        .clicked()
                    {
                        if let Err(e) = app.run_dot_clean() {
                            show_anyhow_error("Error running dot_clean", &e);
                        }
                    }
                }
            });

            let add_games_button = ui
                .button("➕ Add Game(s)")
                .on_hover_text("Add one or more games to the wbfs directory");

            if add_games_button.clicked() {
                app.add_isos();
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ Wiki", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");

                ui.label("•");

                ui.label(format!("Size: {}", Size::from_bytes(app.base_dir_size)));

                let base_dir_name = app.base_dir.file_name().unwrap().to_string_lossy();
                if ui
                    .hyperlink_to(base_dir_name.clone(), "_blank")
                    .on_hover_text(format!("Open the base directory ({base_dir_name})"))
                    .clicked()
                {
                    if let Err(e) = open::that(&app.base_dir) {
                        show_anyhow_error("Error", &anyhow!(e));
                    }
                }

                ui.label("•");

                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}
