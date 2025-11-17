// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    disc_info::DiscInfo,
    games::GameID,
    ui::{self, UiAction},
};
use eframe::egui;

const CARD_WIDTH: f32 = 161.5;
const CARD_HORIZONTAL_SPACE: usize = 181;
const CARD_HEIGHT: f32 = 188.;

pub fn update(ui: &mut egui::Ui, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = available_width as usize / CARD_HORIZONTAL_SPACE;

        if ui_buffers.show_wii {
            ui.heading(format!(
                "🎾 Wii Games: {} found ({})",
                app_state.filtered_wii_games.len(),
                app_state.filtered_wii_games_size
            ));

            ui.add_space(5.);

            for row in app_state.filtered_wii_games.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for game_i in row.iter().copied() {
                        update_game_card(ui, app_state, ui_buffers, game_i);
                    }
                });

                ui.add_space(5.);
            }
        }

        if ui_buffers.show_gc {
            if ui_buffers.show_wii {
                ui.add_space(10.);
            }

            ui.heading(format!(
                "🎲 GameCube Games: {} found ({})",
                app_state.filtered_gc_games.len(),
                app_state.filtered_gc_games_size
            ));

            ui.add_space(5.);

            for row in app_state.filtered_gc_games.chunks(cols) {
                ui.horizontal_top(|ui| {
                    for game_i in row.iter().copied() {
                        update_game_card(ui, app_state, ui_buffers, game_i);
                    }
                });

                ui.add_space(5.);
            }
        }
    });
}

fn update_game_card(
    ui: &mut egui::Ui,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    game_i: u16,
) {
    let game = &app_state.games[game_i as usize];

    let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with id label on the left and size label on the right
            ui.horizontal(|ui| {
                // ID label on the left
                ui.label(format!("🏷  {}", game.id.as_str()));

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(game.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and title
            ui.add(egui::Image::new(&game.image_uri).max_height(96.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(&game.display_title).truncate());

            ui.add_space(10.);

            // Bottom row with buttons

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button
                if ui.button("🗑").on_hover_text("Delete Game").clicked() {
                    ui_buffers.action = Some(UiAction::OpenModal(ui::Modal::DeleteGame(game_i)));
                }

                // Archive button
                if ui
                    .button("📥")
                    .on_hover_text("Archive Game to RVZ or ISO")
                    .clicked()
                {
                    ui_buffers.archiving_game_i = game_i;
                    ui_buffers.choose_archive_path.save_file();
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show Disc Information")
                    .clicked()
                {
                    let game = &app_state.games[game_i as usize];
                    let disc_info = Box::new(DiscInfo::from_game_dir(&game.path).ok());
                    let game_info = Box::new(app_state.get_game_info(game.id));

                    ui_buffers.action = Some(UiAction::OpenModal(ui::Modal::GameInfo(
                        game_i, disc_info, game_info,
                    )));
                }
            });
        });
    });
}
