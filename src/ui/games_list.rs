// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::fill as ph;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(100.))
        .column(Column::auto().at_least(75.))
        .column(Column::remainder())
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.heading("Title");
            });
            header.col(|ui| {
                ui.heading("Console");
            });
            header.col(|ui| {
                ui.heading("Size");
            });
            header.col(|ui| {
                ui.heading("Actions");
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
                            egui::Label::new(if game.is_wii {
                                format!("{} Wii", ph::HAND_DEPOSIT)
                            } else {
                                format!("{} GC", ph::GAME_CONTROLLER)
                            })
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
                                .button(format!("{} Info", ph::INFO))
                                .on_hover_text("Show Game Information")
                                .clicked()
                            {
                                app.send_msg(Message::OpenGameInfo(game_i));
                            }

                            // Archive button
                            if ui
                                .button(format!("{} Archive", ph::BOX_ARROW_DOWN))
                                .on_hover_text("Archive Game to RVZ or ISO")
                                .clicked()
                            {
                                app.send_msg(Message::ArchiveGame(game_i));
                            }

                            // Delete button
                            if ui
                                .button(format!("{} Delete", ph::TRASH))
                                .on_hover_text("Delete Game")
                                .clicked()
                            {
                                app.send_msg(Message::DeleteGame(game_i));
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
