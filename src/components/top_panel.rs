// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, BackgroundMessage};
use crate::components::fake_link::fake_link;
use anyhow::anyhow;
use eframe::egui;
use size::Size;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    let sender = app.inbox.sender();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                // Re-pick base directory button
                if ui.button("📁 Re-pick Base Directory").clicked() {
                    if let Some(new_dir) = rfd::FileDialog::new()
                        .set_title("Select New Base Directory")
                        .pick_folder()
                    {
                        if let Err(e) = app.change_base_dir(new_dir) {
                            let _ = sender.send(BackgroundMessage::Error(e));
                        }
                    }
                }

                // dot_clean button
                if cfg!(target_os = "macos") {
                    if ui
                        .button("👻 Clean MacOS ._ files")
                        .on_hover_text(format!("Run dot_clean in {}", app.base_dir.display()))
                        .clicked()
                    {
                        if let Err(e) = app.run_dot_clean() {
                            let _ = sender.send(BackgroundMessage::Error(e));
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

            // Tests (only debug builds)
            #[cfg(debug_assertions)]
            {
                ui.separator();
                ui.menu_button("🛠 Tests", |ui| {
                    if ui.button("❌ Test Error").clicked() {
                        let _ = sender.send(BackgroundMessage::Error(anyhow!("Test error")));
                    }

                    if ui.button("❌ Test Error 2").clicked() {
                        rfd::MessageDialog::new()
                            .set_title("Test Error 2")
                            .set_level(rfd::MessageLevel::Error)
                            .show();
                    }
                });
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ Wiki", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");

                ui.label("•");

                ui.label(format!("Size: {}", Size::from_bytes(app.base_dir_size)));

                let base_dir_name = app
                    .base_dir
                    .file_name()
                    .map_or_else(
                        // Fallback if file_name() returns None
                        || app.base_dir.to_string_lossy(),
                        // Convert the OsStr to a string if Some
                        |name| name.to_string_lossy(),
                    )
                    .into_owned();

                if fake_link(ui, &base_dir_name)
                    .on_hover_text(format!("Open the base directory ({base_dir_name})"))
                    .clicked()
                {
                    if let Err(e) = open::that(&app.base_dir) {
                        let _ = sender.send(BackgroundMessage::Error(anyhow!(e)));
                    }
                }

                ui.label("•");

                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}
