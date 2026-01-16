// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{
    config::{SortBy, ViewAs},
    hbc_apps, ui,
    ui::{hbc_apps_grid, hbc_apps_list},
};
use eframe::egui::{self, Vec2};
use egui_phosphor::regular as ph;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading(format!(
                "Click on {} to select a Drive/Mount Point",
                ph::HARD_DRIVE
            ));
            return;
        }

        update_top_bar(ui, ctx, frame, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => hbc_apps_grid::update(ui, app),
            ViewAs::List => hbc_apps_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    let current_view_as = app.config.contents.view_as;
    let current_sort_by = app.config.contents.sort_by;

    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    ui.horizontal(move |ui| {
        group.show(ui, |ui| {
            ui.add_space(3.);
            ui.vertical(|ui| {
                ui.add_space(2.);
                ui.label(ph::MAGNIFYING_GLASS);
            });

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
                    egui::Button::new(egui::RichText::new(ph::PLUS).strong().size(18.)),
                )
                .on_hover_text("Add Apps")
                .clicked()
            {
                let paths = ui::dialogs::choose_hbc_apps(frame);
                if !paths.is_empty() {
                    hbc_apps::spawn_install_apps_task(app, paths.into_boxed_slice());
                }
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new(ph::ARROW_CLOCKWISE).strong().size(18.)),
                )
                .on_hover_text("Refresh Apps")
                .clicked()
            {
                app.refresh_hbc_apps();
                app.update_title(ctx);
            }

            ui.add_space(10.);

            group.show(ui, |ui| {
                if ui
                    .selectable_label(current_view_as == ViewAs::List, ph::ROWS)
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    app.save_config();
                }

                if ui
                    .selectable_label(current_view_as == ViewAs::Grid, ph::SQUARES_FOUR)
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
                            format!("MiB {}", ph::CARET_DOWN)
                        } else {
                            format!("MiB {}", ph::CARET_UP)
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
                            format!("A-Z {}", ph::CARET_DOWN)
                        } else {
                            format!("A-Z {}", ph::CARET_UP)
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
