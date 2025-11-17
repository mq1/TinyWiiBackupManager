// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{
    config::{SortBy, ViewAs},
    ui::{hbc_apps_grid, hbc_apps_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on 🖴 to select a Drive/Mount Point");
            return;
        }

        update_top_bar(ui, ctx, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => hbc_apps_grid::update(ui, app),
            ViewAs::List => hbc_apps_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App) {
    let current_view_as = app.config.contents.view_as;
    let current_sort_by = app.config.contents.sort_by;

    ui.horizontal(move |ui| {
        let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(2.);
            ui.label("🔎");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.hbc_apps_filter)
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
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new("✚").size(18.)),
                )
                .on_hover_text("Add Apps")
                .clicked()
            {
                app.choose_hbc_apps.pick_multiple();
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new("⟳").size(18.)),
                )
                .on_hover_text("Refresh Apps")
                .clicked()
            {
                app.refresh_hbc_apps();
                app.update_title(ctx);
            }

            ui.add_space(10.);

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(current_view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    app.save_config();
                }

                if ui
                    .selectable_label(current_view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    app.save_config();
                }
            });

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(
                            current_sort_by,
                            SortBy::SizeAscending | SortBy::SizeDescending
                        ),
                        if current_sort_by == SortBy::SizeDescending {
                            "⚖⏷"
                        } else {
                            "⚖⏶"
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
                            "🗛⏷"
                        } else {
                            "🗛⏶"
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
        });
    });
}
