// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::components::game_info::ui_game_info_window;
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use crate::{
    app::App,
    game::{ConsoleType, Game},
};
use eframe::egui::{self, Button, Image, RichText};
use egui_inbox::UiInboxSender;
use size::Size;
use std::path::Path;

const CARD_SIZE: egui::Vec2 = egui::vec2(170.0, 220.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let cover_dir = app.base_dir.as_ref().unwrap().cover_dir();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // expand horizontally
        ui.set_min_width(ui.available_width());

        let num_columns =
            (ui.available_width() / (CARD_SIZE.x + GRID_SPACING.x * 2.)).max(1.) as usize;

        let filter = &app.console_filter;

        egui::Grid::new("game_grid")
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                let games = app.games.iter_mut();

                for (i, game) in games.enumerate() {
                    if filter.shows_game(game) {
                        ui_game_card(
                            ui,
                            &mut app.inbox.sender(),
                            &app.task_processor,
                            game,
                            &cover_dir,
                        );
                    }

                    if (i + 1) % num_columns == 0 {
                        ui.end_row();
                    }

                    // game info window
                    ui_game_info_window(ui.ctx(), game, &mut app.inbox.sender());
                }
            });
    });
}

fn ui_game_card(
    ui: &mut egui::Ui,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
    game: &mut Game,
    cover_dir: &Path,
) {
    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.set_min_size(CARD_SIZE);

        ui.vertical(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(match game.console {
                    ConsoleType::GameCube => "🎮 GC",
                    ConsoleType::Wii => "🎾 Wii",
                });

                // Verified label on the left
                if game.is_verified {
                    ui.colored_label(egui::Color32::DARK_GREEN, "✅")
                        .on_hover_text("✅ crc32 Verified");
                }

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Size::from_bytes(game.size).to_string());
                });
            });

            // Centered content
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let image = Image::new(format!(
                    "file://{}",
                    cover_dir
                        .join(&game.id_str)
                        .with_extension("png")
                        .to_string_lossy()
                ))
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
                        if ui
                            .add(Button::new("🗑"))
                            .on_hover_text("Remove Game")
                            .clicked()
                            && let Err(e) = game.remove()
                        {
                            let _ = sender.send(e.into());
                        }

                        // Verify crc32 button
                        if ui
                            .add(Button::new("🔎"))
                            .on_hover_text("Verify crc32")
                            .clicked()
                        {
                            game.spawn_verify_task(0, 1, task_processor);
                        }

                        let info_button = ui.add(
                            Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                        );

                        if info_button.clicked() {
                            game.toggle_info();
                        }
                    });
                });
            });
        });
    });
}
