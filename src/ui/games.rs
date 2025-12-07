// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{
    config::{SortBy, ViewAs},
    ui::{games_grid, games_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading(format!(
                "Click on {} to select a Drive/Mount Point",
                egui_phosphor::regular::HARD_DRIVE
            ));
            return;
        }

        update_top_bar(ui, ctx, frame, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => games_grid::update(ui, app),
            ViewAs::List => games_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    let current_view_as = app.config.contents.view_as;
    let current_sort_by = app.config.contents.sort_by;

    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    ui.horizontal(|ui| {
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(3.);
            ui.label(egui_phosphor::regular::MAGNIFYING_GLASS);

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.games_filter)
                        .desired_width(200.)
                        .hint_text("Search by Title/ID"),
                )
                .changed()
            {
                app.update_filtered_games();
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(
                        egui::RichText::new(egui_phosphor::regular::FOLDER_PLUS).size(18.),
                    ),
                )
                .on_hover_text("Add Games Recursively")
                .clicked()
            {
                app.add_games_from_dir(frame);
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new(egui_phosphor::regular::PLUS).size(18.)),
                )
                .on_hover_text("Add Games")
                .clicked()
            {
                app.add_games(frame);
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(
                        egui::RichText::new(egui_phosphor::regular::ARROWS_CLOCKWISE).size(18.),
                    ),
                )
                .on_hover_text("Refresh Games")
                .clicked()
            {
                app.refresh_games();
                app.update_title(ctx);
            }

            ui.add_space(10.);

            group.show(ui, |ui| {
                if ui
                    .selectable_label(
                        current_view_as == ViewAs::List,
                        egui_phosphor::regular::LIST,
                    )
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    app.save_config();
                }

                if ui
                    .selectable_label(
                        current_view_as == ViewAs::Grid,
                        egui_phosphor::regular::SQUARES_FOUR,
                    )
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(
                            current_sort_by,
                            SortBy::SizeAscending | SortBy::SizeDescending
                        ),
                        if current_sort_by == SortBy::SizeDescending {
                            format!(
                                "{}{}",
                                egui_phosphor::regular::SCALES,
                                egui_phosphor::regular::SORT_DESCENDING
                            )
                        } else {
                            format!(
                                "{}{}",
                                egui_phosphor::regular::SCALES,
                                egui_phosphor::regular::SORT_ASCENDING
                            )
                        },
                    )
                    .on_hover_text("Sort by size")
                    .clicked()
                {
                    let sort_by = if current_sort_by == SortBy::SizeAscending {
                        SortBy::SizeDescending
                    } else {
                        SortBy::SizeAscending
                    };

                    app.apply_sorting(sort_by);
                }

                if ui
                    .selectable_label(
                        matches!(
                            current_sort_by,
                            SortBy::NameAscending | SortBy::NameDescending
                        ),
                        if current_sort_by == SortBy::NameDescending {
                            format!(
                                "{}{}",
                                egui_phosphor::regular::TEXT_AA,
                                egui_phosphor::regular::SORT_DESCENDING
                            )
                        } else {
                            format!(
                                "{}{}",
                                egui_phosphor::regular::TEXT_AA,
                                egui_phosphor::regular::SORT_ASCENDING
                            )
                        },
                    )
                    .on_hover_text("Sort by name")
                    .clicked()
                {
                    let sort_by = if current_sort_by == SortBy::NameAscending {
                        SortBy::NameDescending
                    } else {
                        SortBy::NameAscending
                    };

                    app.apply_sorting(sort_by);
                }
            });

            group.show(ui, |ui| {
                ui.checkbox(&mut app.show_gc, egui_phosphor::regular::GAME_CONTROLLER)
                    .on_hover_text("Show GameCube");

                ui.separator();

                ui.checkbox(&mut app.show_wii, egui_phosphor::regular::HAND_DEPOSIT)
                    .on_hover_text("Show Wii");
            });
        });
    });
}
