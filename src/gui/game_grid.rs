// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::gui::game_checks::ui_game_checks;
use crate::gui::game_info::ui_game_info_window;
use crate::messages::BackgroundMessage;
use crate::settings::ArchiveFormat;
use crate::task::TaskProcessor;
use crate::{app::App, game::Game};
use eframe::egui::{self, Button, Image, RichText};
use egui_inbox::UiInboxSender;
use size::Size;
use std::cmp::max;
use std::path::Path;

const MIN_CARD_WIDTH: f32 = 150.0;
const CARD_HEIGHT: f32 = 200.0;
const CARD_PADDING: f32 = 5.0;

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    if let Some(base_dir) = &app.base_dir {
        let cover_dir = base_dir.cover_dir();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();
            ui.set_width(available_width);

            // We don't want to divide by zero, don't we?
            let num_columns = max(
                1,
                (available_width / (MIN_CARD_WIDTH + CARD_PADDING * 2.)) as usize,
            );

            // Calculate the width of each card
            let card_width = (available_width / num_columns as f32) - CARD_PADDING * 4.5;

            // Pre-filter visible games
            let mut visible_games: Vec<_> = app
                .games
                .iter_mut()
                .filter(|g| app.console_filter.shows_game(g))
                .collect();

            egui::Grid::new("game_grid")
                .spacing(egui::Vec2::splat(CARD_PADDING * 2.))
                .show(ui, |ui| {
                    for row in visible_games.chunks_mut(num_columns) {
                        for game in row {
                            ui_game_card(
                                ui,
                                card_width,
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
    card_width: f32,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
    game: &mut Game,
    cover_dir: &Path,
    archive_format: ArchiveFormat,
) {
    let card = egui::Frame::group(ui.style()).corner_radius(5.0);

    card.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(card_width);

        ui.vertical(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(format!("{} {}", game.console.icon(), game.console));

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

                let label = egui::Label::new(RichText::new(&game.title).strong()).truncate();
                ui.add(label);

                ui.add_space(10.);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Remove button
                    if ui
                        .button(egui_phosphor::regular::TRASH)
                        .on_hover_text("Remove Game")
                        .clicked()
                        && let Err(e) = game.remove()
                    {
                        let _ = sender.send(e.into());
                    }

                    // Integrity check button
                    if ui
                        .button(egui_phosphor::regular::MAGNIFYING_GLASS)
                        .on_hover_text("Integrity Check")
                        .clicked()
                    {
                        game.spawn_integrity_check_task(task_processor);
                    }

                    // Archive button
                    if ui
                        .button(egui_phosphor::regular::PACKAGE)
                        .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                        .clicked()
                    {
                        game.spawn_archive_task(task_processor, archive_format);
                    }

                    // Info button
                    if ui
                        .add(
                            Button::new(format!("{} Info", egui_phosphor::regular::INFO))
                                .min_size(egui::vec2(ui.available_width(), 0.0)),
                        )
                        .on_hover_text("Show Game Information")
                        .clicked()
                    {
                        game.toggle_info();
                    }
                });
            });
        });
    });
}
