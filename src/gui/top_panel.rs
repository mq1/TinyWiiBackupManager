// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, View};
use crate::gui::wiiload::ui_wiiload;
use eframe::egui;
use eframe::egui::PopupCloseBehavior;

/// Renders the top menu bar.
pub fn ui_top_panel(ctx: &egui::Context, app: &mut App) {
    let sender = app.inbox.sender();

    egui::TopBottomPanel::top("top_panel")
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.add_space(6.);

            ui.horizontal(|ui| {
                // nav
                ui.selectable_value(
                    &mut app.view,
                    View::Games,
                    format!("{} Games", egui_phosphor::regular::GAME_CONTROLLER),
                )
                .on_hover_text("View your Wii games");

                ui.selectable_value(
                    &mut app.view,
                    View::WiiApps,
                    format!("{} Apps", egui_phosphor::regular::STAR),
                )
                .on_hover_text("View your Homebrew Channel apps");

                ui.selectable_value(
                    &mut app.view,
                    View::OSCWii,
                    format!("{} OSCWii", egui_phosphor::regular::STOREFRONT),
                )
                .on_hover_text("Download apps from OSCWii.org");

                ui.selectable_value(&mut app.view, View::Settings, egui_phosphor::regular::GEAR)
                    .on_hover_text(format!("Open the {} settings", env!("CARGO_PKG_NAME")));

                // Display the total number of games on the right side of the menu bar
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        egui_phosphor::regular::INFO,
                        "https://github.com/mq1/TinyWiiBackupManager/wiki",
                    )
                    .on_hover_text(format!("Open the {} wiki", env!("CARGO_PKG_NAME")));

                    ui.menu_button(egui_phosphor::regular::LIST, |ui| {
                        // remove hint toast
                        app.top_right_toasts.dismiss_all_toasts();

                        // Re-pick base directory button
                        if ui
                            .button(format!(
                                "{} Pick base Drive/Directory",
                                egui_phosphor::regular::FOLDER
                            ))
                            .clicked()
                            && let Err(e) = app.choose_base_dir()
                        {
                            let _ = sender.send(e.into());
                        }

                        // dot_clean button
                        if cfg!(target_os = "macos") {
                            ui.separator();

                            let mut btn = ui.add_enabled(
                                app.base_dir.is_some(),
                                egui::Button::new(format!(
                                    "{} Clean MacOS ._ files",
                                    egui_phosphor::regular::GHOST
                                )),
                            );

                            if let Some(base_dir) = &app.base_dir {
                                btn = btn.on_hover_text(format!("Run dot_clean in {base_dir}"));

                                if btn.clicked()
                                    && let Err(e) = base_dir.run_dot_clean()
                                {
                                    let _ = sender.send(e.into());
                                }
                            }
                        }

                        ui.separator();

                        // Download database button
                        if ui
                            .add_enabled(
                                app.base_dir.is_some(),
                                egui::Button::new(format!(
                                    "{} Download wiitdb.xml",
                                    egui_phosphor::regular::DOWNLOAD_SIMPLE
                                )),
                            )
                            .on_hover_text("Download the latest wiitdb.xml database from GameTDB")
                            .clicked()
                        {
                            app.spawn_download_database_task();
                        }

                        // Download covers option
                        if ui
                            .add_enabled(
                                !app.games.is_empty(),
                                egui::Button::new(format!(
                                    "{} Download Covers",
                                    egui_phosphor::regular::DOWNLOAD_SIMPLE
                                )),
                            )
                            .on_hover_text("Download all covers for all games")
                            .clicked()
                        {
                            app.download_all_covers();
                        }

                        ui.separator();

                        if ui
                            .add_enabled(
                                !app.games.is_empty(),
                                egui::Button::new(format!(
                                    "{} Integrity check (all games)",
                                    egui_phosphor::regular::MAGNIFYING_GLASS
                                )),
                            )
                            .on_hover_text("Check the integrity of all games")
                            .clicked()
                        {
                            app.total_integrity_check();
                        }
                    });

                    if app.view == View::WiiApps {
                        let btn = ui.button(format!("{} Wiiload", egui_phosphor::regular::MAILBOX));
                        let popup = egui::Popup::from_toggle_button_response(&btn)
                            .close_behavior(PopupCloseBehavior::CloseOnClickOutside);
                        popup.show(|ui| {
                            app.top_right_toasts.dismiss_all_toasts();
                            ui_wiiload(ui, app);
                        });
                    }

                    if app.base_dir.is_some() {
                        if app.view == View::Games {
                            let add_games_button = ui
                                .button(format!(
                                    "{} Add Game(s)",
                                    egui_phosphor::regular::PLUS_CIRCLE
                                ))
                                .on_hover_text("Add one or more games to the wbfs directory");

                            if add_games_button.clicked() {
                                app.add_isos();
                            }
                        } else if app.view == View::WiiApps {
                            let add_apps_button = ui
                                .button(format!("{} Add .zip", egui_phosphor::regular::PLUS_CIRCLE))
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
