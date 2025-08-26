// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::cover_manager::{CoverManager, CoverType};
use crate::messages::BackgroundMessage;
use crate::{
    app::App,
    game::{ConsoleType, Game, VerificationStatus},
};
use eframe::egui::{self, Button, Image, RichText};
use size::Size;

const CARD_SIZE: egui::Vec2 = egui::vec2(170.0, 220.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

pub fn ui_game_grid(ui: &mut egui::Ui, app: &mut App) {
    let sender = app.inbox.sender();

    let mut to_remove = None;
    let mut to_open_info = None;
    let mut covers_to_download = Vec::new();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // expand horizontally
        ui.set_min_width(ui.available_width());

        let num_columns =
            (ui.available_width() / (CARD_SIZE.x + GRID_SPACING.x * 2.)).max(1.) as usize;

        let filter = &app.console_filter;

        egui::Grid::new("game_grid")
            .spacing(GRID_SPACING)
            .show(ui, |ui| {
                let mut column_index = 0;

                for (original_index, game) in app.games.iter().enumerate() {
                    if filter.shows_game(game) {
                        let (should_remove, should_open_info, should_download_cover) =
                            ui_game_card(ui, game, &app.cover_manager);
                        if should_remove {
                            to_remove = Some((*game).clone());
                        }
                        if should_open_info {
                            to_open_info = Some(original_index);
                        }
                        if should_download_cover {
                            covers_to_download.push(game.id.clone());
                        }

                        column_index += 1;
                        if column_index % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                }
            });
    });

    // Handle cover downloads after the UI loop
    if let Some(cover_manager) = &app.cover_manager {
        for game_id in covers_to_download {
            cover_manager.queue_download(game_id, CoverType::Cover3D);
        }
    }

    if let Some(game) = to_remove
        && let Err(e) = game.remove()
    {
        let _ = sender.send(BackgroundMessage::Error(e));
    }

    if let Some(index) = to_open_info {
        app.open_game_info(index);
    }
}

fn ui_game_card(
    ui: &mut egui::Ui,
    game: &Game,
    cover_manager: &Option<CoverManager>,
) -> (bool, bool, bool) {
    let mut remove_clicked = false;
    let mut info_clicked = false;
    let mut should_download_cover = false;

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

                // Verification status icon
                match game.get_verification_status() {
                    VerificationStatus::EmbeddedMatch(game) => {
                        ui.label(RichText::new("⚡").color(egui::Color32::from_rgb(255, 200, 0)))
                            .on_hover_text(format!("Embedded hashes match: {}", game.name));
                    }
                    VerificationStatus::FullyVerified(game, _) => {
                        ui.label(RichText::new("✅").color(egui::Color32::DARK_GREEN))
                            .on_hover_text(format!("Fully verified: {}", game.name));
                    }
                    VerificationStatus::Failed(message, _) => {
                        ui.label(RichText::new("❌").color(egui::Color32::DARK_RED))
                            .on_hover_text(message);
                    }
                    _ => {}
                }

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Size::from_bytes(game.size).to_string());
                });
            });

            // Centered content
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Handle cover art
                if let Some(cover_manager) = cover_manager {
                    let cover_path = cover_manager.get_cover_path(&game.id, CoverType::Cover3D);

                    if cover_path.exists() {
                        // Show existing local cover
                        let image = Image::new(format!("file://{}", cover_path.display()))
                            .max_height(128.0)
                            .maintain_aspect_ratio(true);
                        ui.add(image);
                    } else if cover_manager.is_downloading(&game.id, CoverType::Cover3D) {
                        // Show placeholder while downloading
                        ui.allocate_ui(egui::vec2(128.0, 128.0), |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                                ui.label("Downloading cover...");
                            });
                        });
                    } else {
                        // Show placeholder and mark for download
                        ui.allocate_ui(egui::vec2(128.0, 128.0), |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label("📦");
                            });
                        });

                        // Mark for download
                        should_download_cover = true;
                    }
                } else {
                    // No cover manager - show placeholder
                    ui.allocate_ui(egui::vec2(128.0, 128.0), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label("🎮");
                        });
                    });
                }

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

    (remove_clicked, info_clicked, should_download_cover)
}
