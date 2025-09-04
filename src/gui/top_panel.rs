// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::gui::fake_link::fake_link;
use anyhow::anyhow;
use eframe::egui;
use size::Size;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    let sender = app.inbox.sender();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("📄 File", |ui| {
                // remove hint toast
                app.top_left_toasts.dismiss_all_toasts();

                // Re-pick base directory button
                if ui.button("📁 Pick base Drive/Directory").clicked()
                    && let Err(e) = app.choose_base_dir()
                {
                    let _ = sender.send(e.into());
                }

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
            });

            if app.base_dir.is_some() {
                let add_games_button = ui
                    .button("➕ Add Game(s)")
                    .on_hover_text("Add one or more games to the wbfs directory");

                if add_games_button.clicked() {
                    app.add_isos();
                }

                if ui
                    .button("🔎 Integrity check")
                    .on_hover_text("Check the integrity of all games")
                    .clicked()
                {
                    app.total_integrity_check();
                }

                // GameTDB menu
                ui.menu_button("🎮 GameTDB", |ui| {
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
                });
            }

            // Display the total number of games on the right side of the menu bar
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to("ℹ", "https://github.com/mq1/TinyWiiBackupManager/wiki")
                    .on_hover_text("Open the TinyWiiBackupManager wiki");

                if ui.button("⚙").clicked() {
                    app.settings_window_open = !app.settings_window_open;
                }

                if let Some(base_dir) = &app.base_dir {
                    ui.label("•");

                    ui.label(format!("({})", Size::from_bytes(app.base_dir_size)));

                    let base_dir_name = base_dir.name();
                    if fake_link(ui, &base_dir_name)
                        .on_hover_text(format!("Open the base directory ({base_dir_name})"))
                        .clicked()
                        && let Err(e) = base_dir.open()
                    {
                        let _ = sender.send(e.into());
                    }
                }

                // Show game count if the base_dir is some
                if app.base_dir.is_some() {
                    ui.label(format!("{} games in", app.games.len()));
                }
            });
        });
    });
}
