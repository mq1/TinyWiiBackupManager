// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::{SortBy, ViewAs},
    ui::{games_grid, games_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on ☰ ⏵ 🖴 to choose a Drive or Directory");
            return;
        }

        update_top_bar(ui, app, ctx);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => games_grid::update(ui, app),
            ViewAs::List => games_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, app: &mut App, ctx: &egui::Context) {
    ui.horizontal(move |ui| {
        let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(2.);
            ui.label("🔎");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.game_search)
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
                    egui::Button::new(egui::RichText::new("✚").size(18.)),
                )
                .on_hover_text("Add Games")
                .clicked()
            {
                app.choose_games.open();
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new("⟳").size(18.)),
                )
                .on_hover_text("Refresh Games")
                .clicked()
            {
                app.refresh_games(ctx);
            }

            ui.add_space(10.);

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    if let Err(e) = app.config.write() {
                        app.toasts.error(e.to_string());
                    }
                }

                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    if let Err(e) = app.config.write() {
                        app.toasts.error(e.to_string());
                    }
                }
            });

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(
                            app.config.contents.sort_by,
                            SortBy::SizeAscending | SortBy::SizeDescending
                        ),
                        if app.config.contents.sort_by == SortBy::SizeDescending {
                            "⚖⏷"
                        } else {
                            "⚖⏶"
                        },
                    )
                    .on_hover_text("Sort by size")
                    .clicked()
                {
                    app.config.contents.sort_by =
                        if app.config.contents.sort_by == SortBy::SizeAscending {
                            SortBy::SizeDescending
                        } else {
                            SortBy::SizeAscending
                        };
                    app.apply_sorting();
                }

                if ui
                    .selectable_label(
                        matches!(
                            app.config.contents.sort_by,
                            SortBy::NameAscending | SortBy::NameDescending
                        ),
                        if app.config.contents.sort_by == SortBy::NameDescending {
                            "🗛⏷"
                        } else {
                            "🗛⏶"
                        },
                    )
                    .on_hover_text("Sort by name")
                    .clicked()
                {
                    app.config.contents.sort_by =
                        if app.config.contents.sort_by == SortBy::NameAscending {
                            SortBy::NameDescending
                        } else {
                            SortBy::NameAscending
                        };
                    app.apply_sorting();
                }
            });

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .checkbox(&mut app.show_gc, "🎲")
                    .on_hover_text("Show GameCube")
                    .changed()
                {
                    app.update_filtered_games();
                }

                ui.separator();

                if ui
                    .checkbox(&mut app.show_wii, "🎾")
                    .on_hover_text("Show Wii")
                    .changed()
                {
                    app.update_filtered_games();
                }
            });
        });
    });
}
