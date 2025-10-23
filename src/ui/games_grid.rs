// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{App, GameInfoData},
    disc_info::DiscInfo,
    games::Game,
    wiitdb::{self},
};
use eframe::egui::{self, Vec2};
use egui_file_dialog::FileDialog;
use egui_notify::Toasts;
use std::path::{Path, PathBuf};

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
                            &mut app.game_info,
                            &mut app.archiving_game,
                            &mut app.choose_archive_path,
                            &app.config.contents.mount_point,
                            &mut app.wiitdb,
                            &mut app.toasts,
                        );
                    }

                    ui.end_row();
                }
            });
    });
}

#[allow(clippy::too_many_arguments)]
fn view_game_card(
    ui: &mut egui::Ui,
    game: &Game,
    removing_game: &mut Option<Game>,
    info: &mut Option<GameInfoData>,
    archiving_game: &mut Option<PathBuf>,
    choose_archive_path: &mut FileDialog,
    mount_point: &Path,
    wiitdb: &mut Option<wiitdb::Datafile>,
    toasts: &mut Toasts,
) {
    let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(if game.is_wii { "🎾 Wii" } else { "🎲 GC" });

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

                // Archive button
                if ui
                    .button("📥")
                    .on_hover_text("Archive Game to RVZ or ISO")
                    .clicked()
                {
                    *archiving_game = Some(game.path.clone());
                    choose_archive_path.save_file();
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show Disc Information")
                    .clicked()
                {
                    let game = game.clone();
                    let disc_info = DiscInfo::from_game_dir(&game.path).map_err(|e| e.to_string());

                    if wiitdb.is_none() {
                        match wiitdb::Datafile::load(mount_point) {
                            Ok(new) => {
                                *wiitdb = Some(new);
                            }
                            Err(e) => {
                                toasts.error(e.to_string());
                            }
                        }
                    }

                    let game_info = wiitdb
                        .as_ref()
                        .and_then(|wiitdb| wiitdb.get_game_info(&game.id).cloned())
                        .ok_or("Game not found in wiitdb".to_string());

                    *info = Some((game, disc_info, game_info));
                }
            });
        });
    });
}
