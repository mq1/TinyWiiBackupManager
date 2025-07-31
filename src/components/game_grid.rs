// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, game::Game};
use eframe::egui::{self, Image, RichText};

const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 220.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let mut to_remove = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        let num_columns = ((ui.available_width() / CARD_SIZE.x) as usize).max(1);

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
        app.remove_game(&game);
    }
}

fn ui_game_card(ui: &mut egui::Ui, game: &Game) -> bool {
    let mut remove_clicked = false;

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.vertical_centered(|ui| {
            let image = Image::new(format!(
                "https://art.gametdb.com/wii/cover/US/{}.png",
                game.id
            ))
            .max_height(140.0)
            .maintain_aspect_ratio(true)
            .show_loading_spinner(true);

            ui.add(image);
            ui.add_space(5.0);

            ui.label(RichText::new(&game.display_title).strong());
            ui.label(
                RichText::new(format!("ID: {}", game.id))
                    .monospace()
                    .size(12.0),
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                remove_clicked = ui.button("🗑 Remove").clicked();
            });
        });
    });

    remove_clicked
}