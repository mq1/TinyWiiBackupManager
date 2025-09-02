// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::components::game_checks::ui_game_checks;
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

const CARD_WIDTH: f32 = 188.5;
const CARD_HEIGHT: f32 = 220.0;
const GRID_SPACING: f32 = 10.0;

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    if let Some(base_dir) = &app.base_dir {
        let cover_dir = base_dir.cover_dir();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // expand horizontally
            ui.set_min_width(ui.available_width());

            let num_columns =
                (ui.available_width() / (CARD_WIDTH + GRID_SPACING / 2.)).max(1.) as usize;

            let filter = &app.console_filter;

            egui::Grid::new("game_grid")
                .min_col_width(CARD_WIDTH)
                .min_row_height(CARD_HEIGHT)
                .spacing(egui::Vec2::splat(GRID_SPACING))
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
            ui.vertical(|ui| {
                // Top row with console label on the left and size label on the right
                ui.horizontal(|ui| {
                    // Console label on the left
                    ui.label(match game.console {
                        ConsoleType::GameCube => "🎮 GC",
                        ConsoleType::Wii => "🎾 Wii",
                    });

                    // Game checks on the left
                    ui_game_checks(ui, game);

                    // Size label on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(Size::from_bytes(game.size).to_string());
                    });
                });

                // Centered content
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let image = Image::new(game.get_local_cover_uri(cover_dir))
                        .max_height(128.0)
                        .maintain_aspect_ratio(true)
                        .show_loading_spinner(true);
                    ui.add(image);

                    ui.add_space(5.);

                    let label =
                        egui::Label::new(RichText::new(&game.display_title).strong()).truncate();
                    ui.add(label);
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

                            // Integrity check button
                            if ui
                                .add(Button::new("🔎"))
                                .on_hover_text("Integrity Check")
                                .clicked()
                            {
                                game.spawn_integrity_check_task(task_processor);

                                // Does nothing, but increments the task counter
                                task_processor.spawn_task(move |ui_sender| {
                                    let _ = ui_sender.send(BackgroundMessage::ClearStatus);
                                    Ok(())
                                });
                            }

                            // Archive button
                            if ui
                                .add(Button::new("📦"))
                                .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                                .clicked()
                            {
                                game.spawn_archive_task(task_processor);

                                // Does nothing, but increments the task counter
                                task_processor.spawn_task(move |ui_sender| {
                                    let _ = ui_sender.send(BackgroundMessage::ClearStatus);
                                    Ok(())
                                });
                            }

                            // Info button
                            if ui
                                .add(
                                    Button::new("ℹ Info")
                                        .min_size(egui::vec2(ui.available_width(), 0.0)),
                                )
                                .on_hover_text("Show Game Info")
                                .clicked()
                            {
                                game.toggle_info();
                            }
                        });
                    });
                });
            });
        });
    }
}
