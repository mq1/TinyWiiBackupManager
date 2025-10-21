// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::{app::App, disc_info::DiscInfo, games::Game};
use eframe::egui::{self, Vec2};

const CARD_WIDTH: f32 = 161.5;
const CARD_HEIGHT: f32 = 188.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = (available_width / (CARD_WIDTH + 20.)).floor() as usize;

        egui::Grid::new("games")
            .num_columns(cols)
            .spacing(Vec2::splat(8.))
            .show(ui, |ui| {
                for row in app.filtered_games.chunks(cols) {
                    for game in row {
                        view_game_card(
                            ui,
                            game,
                            &mut app.removing_game,
                            &mut app.disc_info,
                            &app.data_dir,
                        );
                    }

                    ui.end_row();
                }
            });
    });
}

fn view_game_card(
    ui: &mut egui::Ui,
    game: &Game,
    removing_game: &mut Option<Game>,
    disc_info: &mut Option<(String, DiscInfo)>,
    data_dir: &Path,
) {
    ui.group(|ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(if game.is_wii { "🎾 Wii" } else { "◼ GC" });

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
                // Remove button
                if ui.button("🗑").on_hover_text("Remove Game").clicked() {
                    *removing_game = Some(game.clone());
                }

                // Integrity check button
                ui.button("✅").on_hover_text("Integrity Check").clicked();

                // Archive button
                ui.button("📥")
                    .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                    .clicked();

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show Disc Information")
                    .clicked()
                {
                    let info = DiscInfo::from_game_dir(&game.path, &data_dir).unwrap_or_default();
                    *disc_info = Some((game.display_title.clone(), info));
                }
            });
        });
    });
}
