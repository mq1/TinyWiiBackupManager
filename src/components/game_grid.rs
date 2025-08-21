// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, error_handling::show_anyhow_error, game::Game};
use eframe::egui::{self, Button, Image, RichText};

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

        egui::Grid::new("game_grid")
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                for (i, game) in app.games.iter().enumerate() {
                    let (should_remove, should_open_info) = ui_game_card(ui, game);
                    if should_remove {
                        to_remove = Some(game.clone());
                    }
                    if should_open_info {
                        to_open_info = Some(game.path.clone());
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

    if let Some(path) = to_open_info {
        app.open_game_info(path);
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
            // Console label on the left
            let console = if game.is_gc { "🎮 GC" } else { "🎾 Wii" };
            ui.label(console);

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

            // Spacer
            let button_row_height = ui.style().spacing.interact_size.y;
            let separator_and_padding = 1.0 + ui.style().spacing.item_spacing.y * 2.0;
            let needed_height = button_row_height + separator_and_padding;
            ui.add_space((ui.available_height() - needed_height).max(0.0));

            ui.separator();

            // Buttons
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    remove_clicked = ui.add(Button::new("🗑")).on_hover_text("Remove Game").clicked();
                    let info_button = Button::new("ℹ Info").min_size(ui.available_size());
                    info_clicked = ui.add(info_button).clicked();
                });
            });
        });
    });

    (remove_clicked, info_clicked)
}
