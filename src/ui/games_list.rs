// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, disc_info::DiscInfo, wiitdb};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(100.))
        .column(Column::auto().at_least(75.))
        .column(Column::remainder())
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.heading("✏ Title");
            });
            header.col(|ui| {
                ui.heading("🎮 Console");
            });
            header.col(|ui| {
                ui.heading("⚖ Size");
            });
            header.col(|ui| {
                ui.heading("☞ Actions");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().spacing.item_spacing.y = 0.0;

            for game in app
                .filtered_wii_games
                .iter()
                .map(|g| (true, g)) // true for Wii, false for GC
                .chain(app.filtered_gc_games.iter().map(|g| (false, g)))
            {
                body.row(26., |mut row| {
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&game.1.display_title).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(
                            egui::Label::new(if game.0 { "🎾 Wii" } else { "🎲 GC" }).truncate(),
                        );
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(game.1.size.to_string()).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui
                                .button("ℹ Info")
                                .on_hover_text("Show Game Information")
                                .clicked()
                            {
                                let disc_info = DiscInfo::from_game_dir(&game.1.path)
                                    .map_err(|e| e.to_string());

                                if app.wiitdb.is_none() {
                                    match wiitdb::Datafile::load(&app.config.contents.mount_point) {
                                        Ok(new) => {
                                            app.wiitdb = Some(new);
                                        }
                                        Err(e) => {
                                            app.toasts.error(e.to_string());
                                        }
                                    }
                                }

                                let game_info = app
                                    .wiitdb
                                    .as_ref()
                                    .and_then(|wiitdb| wiitdb.get_game_info(&game.1.id).cloned())
                                    .ok_or("Game not found in wiitdb".to_string());

                                app.game_info = Some((game.1.clone(), disc_info, game_info));
                            }

                            // Archive button
                            if ui
                                .button("📥 Archive")
                                .on_hover_text("Archive Game to RVZ or ISO")
                                .clicked()
                            {
                                app.archiving_game = Some(game.1.path.clone());
                                app.choose_archive_path.save_file();
                            }

                            // Delete button
                            if ui.button("🗑 Delete").on_hover_text("Delete Game").clicked() {
                                app.deleting_game = Some(game.1.clone());
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
