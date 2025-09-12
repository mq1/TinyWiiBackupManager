// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::gui::game_checks::ui_game_checks;
use crate::gui::game_info::ui_game_info_window;
use crate::messages::BackgroundMessage;
use crate::settings::ArchiveFormat;
use crate::task::TaskProcessor;
use crate::{app::App, game::Game};
use eframe::egui::{self, Image, RichText};
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
            ui.set_min_width(ui.available_width());

            let num_columns =
                (ui.available_width() / (CARD_WIDTH + GRID_SPACING / 2.)).max(1.) as usize;

            let filter = &app.console_filter;

            // Pre-filter visible games
            let mut visible_games: Vec<_> = app
                .games
                .iter_mut()
                .filter(|g| filter.shows_game(g))
                .collect();

            egui::Grid::new("game_grid")
                .min_col_width(CARD_WIDTH)
                .min_row_height(CARD_HEIGHT)
                .spacing(egui::Vec2::splat(GRID_SPACING))
                .show(ui, |ui| {
                    for row in visible_games.chunks_mut(num_columns) {
                        for game in row {
                            ui_game_card(
                                ui,
                                &mut app.inbox.sender(),
                                &app.task_processor,
                                game,
                                &cover_dir,
                                app.settings.archive_format,
                            );
                            ui_game_info_window(ui.ctx(), game, &mut app.inbox.sender());
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

fn ui_game_card(
    ui: &mut egui::Ui,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
    game: &mut Game,
    cover_dir: &Path,
    archive_format: ArchiveFormat,
) {
    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.vertical(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(game.console.as_ref());

                // Game checks on the left
                ui_game_checks(ui, game);

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Size::from_bytes(game.size).to_string());
                });
            });

            // Centered content
            ui.vertical_centered_justified(|ui| {
                let image = Image::new(game.get_local_cover_uri(cover_dir))
                    .max_height(128.0)
                    .maintain_aspect_ratio(true)
                    .show_loading_spinner(true);
                ui.add(image);

                ui.add_space(5.);

                let label =
                    egui::Label::new(RichText::new(&game.display_title).strong()).truncate();
                ui.add(label);

                ui.add_space(10.);

                ui.horizontal(|ui| {
                    // We center the buttons manually
                    // We could use egui_flex, but it's overkill for this simple case
                    ui.add_space(32.);

                    // Info button
                    if ui.button("ℹ").on_hover_text("Show Game Info").clicked() {
                        game.toggle_info();
                    }

                    // Archive button
                    if ui
                        .button("📦")
                        .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                        .clicked()
                    {
                        game.spawn_archive_task(task_processor, archive_format);
                    }

                    // Integrity check button
                    if ui.button("🔎").on_hover_text("Integrity Check").clicked() {
                        game.spawn_integrity_check_task(task_processor);
                    }

                    // Remove button
                    if ui.button("🗑").on_hover_text("Remove Game").clicked()
                        && let Err(e) = game.remove()
                    {
                        let _ = sender.send(e.into());
                    }
                });
            });
        });
    });
}
