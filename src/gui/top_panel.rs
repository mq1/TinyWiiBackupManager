// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, View};
use eframe::egui;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    let sender = app.inbox.sender();

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.add_space(6.);

            ui.horizontal(|ui| {
                // nav
                ui.selectable_value(&mut app.view, View::Games, "Games");
                ui.selectable_value(&mut app.view, View::WiiApps, "Apps");

                // Display the total number of games on the right side of the menu bar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("ℹ", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                        .on_hover_text("Open the TinyWiiBackupManager wiki");

                    if ui.button("⚙").clicked() {
                        app.settings_window_open = !app.settings_window_open;
                    }

                    ui.menu_button("☰", |ui| {
                        // remove hint toast
                        app.top_left_toasts.dismiss_all_toasts();

                        // Re-pick base directory button
                        if ui.button("📁 Pick base Drive/Directory").clicked()
                            && let Err(e) = app.choose_base_dir()
                        {
                            let _ = sender.send(e.into());
                        }

                        ui.separator();

                        // dot_clean button
                        if cfg!(target_os = "macos")
                            && let Some(base_dir) = &app.base_dir
                            && ui
                                .button("👻 Clean MacOS ._ files")
                                .on_hover_text(format!("Run dot_clean in {base_dir}"))
                                .clicked()
                            && let Err(e) = base_dir.run_dot_clean()
                        {
                            let _ = sender.send(e.into());
                        }

                        ui.separator();

                        // Download database button
                        if ui
                            .button("📥 Download wiitdb.xml")
                            .on_hover_text("Download the latest wiitdb.xml database from GameTDB")
                            .clicked()
                        {
                            app.spawn_download_database_task();
                        }

                        // Download covers option
                        if ui
                            .button("📥 Download Covers")
                            .on_hover_text("Download all covers for all games")
                            .clicked()
                        {
                            app.download_all_covers();
                        }

                        ui.separator();

                        if ui
                            .button("🔎 Integrity check (all games)")
                            .on_hover_text("Check the integrity of all games")
                            .clicked()
                        {
                            app.total_integrity_check();
                        }
                    });

                    if app.base_dir.is_some() {
                        if app.view == View::Games {
                            let add_games_button = ui
                                .button("➕ Add Game(s)")
                                .on_hover_text("Add one or more games to the wbfs directory");

                            if add_games_button.clicked() {
                                app.add_isos();
                            }
                        } else if app.view == View::WiiApps {
                            let add_apps_button = ui
                                .button("➕ Add App(s)")
                                .on_hover_text("Add one or more (.zip) apps to the apps directory");

                            if add_apps_button.clicked() {
                                app.add_wiiapps();
                            }
                        }
                    }
                });
            });
        });
}
