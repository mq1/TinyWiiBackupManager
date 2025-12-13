// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use eframe::egui;
use egui_phosphor::fill as ph;

const CARD_WIDTH: f32 = 161.5;
const CARD_HORIZONTAL_SPACE: usize = 181;
const CARD_HEIGHT: f32 = 188.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = available_width as usize / CARD_HORIZONTAL_SPACE;

        if app.show_wii {
            ui.heading(format!(
                "{} Wii Games: {} found ({})",
                ph::HAND_DEPOSIT,
                app.filtered_wii_games.len(),
                app.filtered_wii_games_size
            ));

            ui.add_space(5.);

            for row in app.filtered_wii_games.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for game_i in row.iter().copied() {
                        update_game_card(ui, app, game_i);
                    }
                });

                ui.add_space(5.);
            }
        }

        if app.show_gc {
            if app.show_wii {
                ui.add_space(10.);
            }

            ui.heading(format!(
                "{} GameCube Games: {} found ({})",
                ph::GAME_CONTROLLER,
                app.filtered_gc_games.len(),
                app.filtered_gc_games_size
            ));

            ui.add_space(5.);

            for row in app.filtered_gc_games.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for game_i in row.iter().copied() {
                        update_game_card(ui, app, game_i);
                    }
                });

                ui.add_space(5.);
            }
        }
    });
}

fn update_game_card(ui: &mut egui::Ui, app: &App, game_i: u16) {
    let game = &app.games[game_i as usize];

    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with id label on the left and size label on the right
            ui.horizontal(|ui| {
                // ID label on the left
                ui.label(format!("{}  {}", ph::TAG, game.id.as_str()));

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(game.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and title
            ui.add(egui::Image::new(&game.image_uri).max_height(96.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(egui::RichText::new(&game.display_title).strong()).truncate());

            ui.add_space(10.);

            // Bottom row with buttons

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button
                if ui.button(ph::TRASH).on_hover_text("Delete Game").clicked() {
                    app.send_msg(Message::DeleteGame(game_i));
                }

                // Archive button
                if ui
                    .button(ph::BOX_ARROW_DOWN)
                    .on_hover_text("Archive Game to RVZ or ISO")
                    .clicked()
                {
                    app.send_msg(Message::ArchiveGame(game_i));
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new(format!("{} Info", ph::INFO))
                            .min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show Disc Information")
                    .clicked()
                {
                    app.send_msg(Message::OpenGameInfo(game_i));
                }
            });
        });
    });
}
