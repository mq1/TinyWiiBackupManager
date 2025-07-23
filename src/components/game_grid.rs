// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, game::Game};
use eframe::egui::{self, Image, RichText};
use std::sync::Arc; // Import Arc

// Constants for grid layout
const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 240.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

/// Renders the grid of available games.
pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    // Use Option<Arc<Game>> to store the game to be removed
    let mut game_to_remove: Option<Arc<Game>> = None;
    let num_columns = (ui.available_width() / CARD_SIZE.x).floor() as usize;

    // Create a scrollable area for the game grid
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("games_grid")
            .num_columns(num_columns)
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                // Iterate over games and render cards
                for (i, game) in app.games.iter().enumerate() {
                    // Pass a clone of the Arc to the card for removal
                    ui_game_card(ui, game, || game_to_remove = Some(game.clone()));

                    // End the grid row after the specified number of columns
                    if (i + 1) % num_columns == 0 {
                        ui.end_row();
                    }
                }
            });
    });

    // If a game was marked for removal, call the remove function
    if let Some(game) = game_to_remove {
        app.remove_game(&game);
    }
}

/// Renders a single game card.
fn ui_game_card(ui: &mut egui::Ui, game: &Arc<Game>, on_remove: impl FnOnce()) { // Accept Arc<Game>
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.vertical_centered(|ui| {
            // Display the game cover image
            let image = Image::new(format!(
                "https://art.gametdb.com/wii/cover/EN/{}.png",
                game.id
            ))
                .max_height(140.0)
                .maintain_aspect_ratio(true)
                .show_loading_spinner(true);
            ui.add(image);

            // Add some spacing
            ui.add_space(5.0);

            // Display the game title and ID
            ui.label(RichText::new(&game.display_title).strong());
            ui.label(
                RichText::new(format!("ID: {}", game.id))
                    .monospace()
                    .size(12.0),
            );

            // Spacer to push the remove button to the bottom of the card
            ui.add_space(ui.available_height() - 35.0); // Adjust for button height

            // Remove button
            if ui.button("🗑 Remove").clicked() {
                on_remove(); // Trigger the removal callback
            }
        });
    });
}