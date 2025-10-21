// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::{SortBy, ViewAs},
    ui::{hbc_apps_grid, hbc_apps_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on ☰ ⏵ 🖴 to choose a Drive or Directory");
            return;
        }

        view_top_bar(ui, app, ctx);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => hbc_apps_grid::update(ui, app),
            ViewAs::List => hbc_apps_list::update(ui, app),
        }
    });
}

fn view_top_bar(ui: &mut egui::Ui, app: &mut App, ctx: &egui::Context) {
    ui.horizontal(move |ui| {
        ui.group(|ui| {
            ui.label(egui::RichText::new("🔎").size(15.5));

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.hbc_app_search)
                        .desired_width(200.)
                        .hint_text("Search by Name"),
                )
                .changed()
            {
                app.update_filtered_hbc_apps();
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_sized(
                    Vec2::splat(32.),
                    egui::Button::new(egui::RichText::new("✚").size(18.)),
                )
                .on_hover_text("Add Apps")
                .clicked()
            {
                app.choose_hbc_apps.pick_multiple();
            }

            if ui
                .add_sized(
                    Vec2::splat(32.),
                    egui::Button::new(egui::RichText::new("⟳").size(18.)),
                )
                .on_hover_text("Refresh Apps")
                .clicked()
            {
                app.refresh_hbc_apps(ctx);
            }

            ui.add_space(10.);

            ui.group(|ui| {
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

            ui.group(|ui| {
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
        });
    });
}
