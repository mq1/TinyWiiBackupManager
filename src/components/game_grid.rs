// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{
    app::App, components::console_filter::ConsoleFilter, error_handling::show_anyhow_error,
    game::Game,
};
use eframe::egui::{self, Button, Image, RichText};
use size::Size;

const CARD_SIZE: egui::Vec2 = egui::vec2(170.0, 220.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let mut to_remove = None;
    let mut to_open_info = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // expand horizontally
        ui.set_min_width(ui.available_width());

        let num_columns =
            (ui.available_width() / (CARD_SIZE.x + GRID_SPACING.x * 2.)).max(1.) as usize;

        let ConsoleFilter { show_wii, show_gc } = app.console_filter;

        let games = if show_wii && show_gc {
            &app.games
        } else if show_wii {
            &app.wii_games
        } else if show_gc {
            &app.gc_games
        } else {
            return;
        };

        egui::Grid::new("game_grid")
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                for (i, game) in games.iter().enumerate() {
                    let (should_remove, should_open_info) = ui_game_card(ui, game);
                    if should_remove {
                        to_remove = Some(game.clone());
                    }
                    if should_open_info {
                        to_open_info = Some(i);
                    }

                    if (i + 1) % num_columns == 0 {
                        ui.end_row();
                    }
                }
            });
    });

    if let Some(game) = to_remove {
        if let Err(e) = game.remove() {
            show_anyhow_error("Failed to remove game", &e);
        }
    }

    if let Some(index) = to_open_info {
        app.open_game_info(index);
    }
}

fn ui_game_card(ui: &mut egui::Ui, game: &Game) -> (bool, bool) {
    let mut remove_clicked = false;
    let mut info_clicked = false;

    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.set_min_size(CARD_SIZE);

        ui.vertical(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                let console = if game.is_gc { "🎮 GC" } else { "🎾 Wii" };
                ui.label(console);

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("💾 {}", Size::from_bytes(game.size)));
                });
            });

            // Centered content
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let image = Image::new(&game.image_url)
                    .max_height(128.0)
                    .maintain_aspect_ratio(true)
                    .show_loading_spinner(true);
                ui.add(image);

                ui.add_space(5.);

                ui.label(RichText::new(&game.display_title).strong());
            });

            // Actions
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let remove_response = ui.add(Button::new("🗑")).on_hover_text("Remove Game");
                        remove_clicked = remove_response.clicked();

                        let info_button = ui.add(
                            Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                        );
                        info_clicked = info_button.clicked();
                    });
                });
            });
        });
    });

    (remove_clicked, info_clicked)
}
