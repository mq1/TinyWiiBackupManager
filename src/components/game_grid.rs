// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, error_handling::show_anyhow_error, game::Game};
use eframe::egui::{self, Image, RichText};

const CARD_SIZE: egui::Vec2 = egui::vec2(170.0, 220.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let mut to_remove = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // expand horizontally
        ui.set_min_width(ui.available_width());

        let num_columns =
            (ui.available_width() / (CARD_SIZE.x + GRID_SPACING.x * 2.)).max(1.) as usize;

        egui::Grid::new("game_grid")
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                for (i, game) in app.games.iter().enumerate() {
                    if ui_game_card(ui, game) {
                        to_remove = Some(game.clone());
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
}

fn ui_game_card(ui: &mut egui::Ui, game: &Game) -> bool {
    let mut remove_clicked = false;

    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);

        ui.vertical_centered(|ui| {
            let image = Image::new(&game.image_url)
                .max_height(128.0)
                .maintain_aspect_ratio(true)
                .show_loading_spinner(true);

            ui.add(image);

            ui.add_space(5.);

            ui.label(RichText::new(&game.display_title).strong());

            // horizontal and centered
            ui.horizontal(|ui| {
                ui.add_space(20.);
                let console = if game.is_gc { "🎮 GC" } else { "🎾 Wii" };
                ui.label(RichText::new(console).strong());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(20.);
                    ui.label(RichText::new(format!("ID: {}", game.id)).monospace());
                });
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(7.);
                    ui.hyperlink_to(
                        "🌐 GameTDB",
                        format!("https://www.gametdb.com/Wii/{}", game.id),
                    );
                    ui.add_space(5.);
                    remove_clicked = ui.button("🗑 Remove").clicked();
                });
                ui.separator();
            });
        });
    });

    remove_clicked
}
