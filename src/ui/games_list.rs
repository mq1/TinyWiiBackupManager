// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{
    disc_info::DiscInfo,
    ui::{self},
};
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

            let iterator = match (app.show_wii, app.show_gc) {
                (true, true) => app.filtered_games.iter().copied(),
                (true, false) => app.filtered_wii_games.iter().copied(),
                (false, true) => app.filtered_gc_games.iter().copied(),
                (false, false) => [].iter().copied(),
            };

            for game_i in iterator {
                let game = &app.games[game_i as usize];

                body.row(26., |mut row| {
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&game.display_title).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(
                            egui::Label::new(if game.is_wii { "🎾 Wii" } else { "🎲 GC" })
                                .truncate(),
                        );
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(game.size.to_string()).truncate());
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
                                let disc_info = Box::new(DiscInfo::from_game_dir(&game.path).ok());
                                let game_info = Box::new(app.get_game_info(game.id));

                                app.send_msg(Message::OpenModal(ui::Modal::GameInfo(
                                    game_i, disc_info, game_info,
                                )));
                            }

                            // Archive button
                            if ui
                                .button("📥 Archive")
                                .on_hover_text("Archive Game to RVZ or ISO")
                                .clicked()
                            {
                                app.send_msg(Message::ArchiveGame(game_i));
                            }

                            // Delete button
                            if ui.button("🗑 Delete").on_hover_text("Delete Game").clicked() {
                                app.send_msg(Message::OpenModal(ui::Modal::DeleteGame(game_i)));
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
