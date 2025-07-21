// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::game::Game;
use egui::{Image, RichText};

const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 240.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

/// Renders the grid of available games
pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let mut game_index_to_remove = None;
    let num_columns = (ui.available_width() / CARD_SIZE.x).floor() as usize;

    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("games_grid")
            .num_columns(num_columns)
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                // Store games in a temporary variable to avoid borrowing self
                let games = &app.games;
                for (i, game) in games.iter().enumerate() {
                    ui_game_card(ui, game, || {
                        game_index_to_remove = Some(i);
                    });

                    if (i + 1) % num_columns == 0 {
                        ui.end_row();
                    }
                }
            });
    });

    if let Some(index) = game_index_to_remove {
        let game = app.games[index].clone();
        app.remove_game(&game);
    }
}

/// Renders a single game card (now a static method)
fn ui_game_card(ui: &mut egui::Ui, game: &Game, on_remove: impl FnOnce()) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.vertical_centered(|ui| {
            // Game cover image
            let image = Image::new(format!(
                "https://art.gametdb.com/wii/cover/EN/{}.png",
                game.id
            ))
            .max_height(140.0)
            .maintain_aspect_ratio(true)
            .show_loading_spinner(true);
            ui.add(image);

            // Game info
            ui.add_space(5.0);
            ui.label(RichText::new(&game.display_title).strong());
            ui.label(
                RichText::new(format!("ID: {}", game.id))
                    .monospace()
                    .size(12.0),
            );

            // Spacer to push button to bottom
            ui.add_space(ui.available_height() - 35.0);

            // Remove button
            if ui.button("🗑 Remove").clicked() {
                on_remove();
            }
        });
    });
}
