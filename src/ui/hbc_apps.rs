// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    config::{SortBy, ViewAs},
    ui::{UiAction, hbc_apps_grid, hbc_apps_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app_state.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on 🖴 to select a Drive/Mount Point");
            return;
        }

        update_top_bar(ui, app_state, ui_buffers);
        ui.add_space(10.);

        match app_state.config.contents.view_as {
            ViewAs::Grid => hbc_apps_grid::update(ui, app_state, ui_buffers),
            ViewAs::List => hbc_apps_list::update(ui, app_state, ui_buffers),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    ui.horizontal(move |ui| {
        let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(2.);
            ui.label("🔎");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut ui_buffers.hbc_apps_filter)
                        .desired_width(200.)
                        .hint_text("Search by Name"),
                )
                .changed()
            {
                ui_buffers.action = Some(UiAction::ApplyFilterHbcApps);
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
                ui_buffers.action = Some(UiAction::AddHbcApps);
            }

            if ui
                .add_sized(
                    Vec2::splat(34.),
                    egui::Button::new(egui::RichText::new("⟳").size(18.)),
                )
                .on_hover_text("Refresh Apps")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::TriggerRefreshHbcApps);
            }

            ui.add_space(10.);

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(app_state.config.contents.view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    ui_buffers.config.contents.view_as = ViewAs::List;
                    ui_buffers.action = Some(UiAction::WriteConfig);
                }

                if ui
                    .selectable_label(app_state.config.contents.view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    ui_buffers.config.contents.view_as = ViewAs::Grid;
                    ui_buffers.action = Some(UiAction::WriteConfig);
                }
            });

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(
                        matches!(
                            app_state.config.contents.sort_by,
                            SortBy::SizeAscending | SortBy::SizeDescending
                        ),
                        if app_state.config.contents.sort_by == SortBy::SizeDescending {
                            "⚖⏷"
                        } else {
                            "⚖⏶"
                        },
                    )
                    .on_hover_text("Sort by size")
                    .clicked()
                {
                    let sort_by = if app_state.config.contents.sort_by == SortBy::SizeAscending {
                        SortBy::SizeDescending
                    } else {
                        SortBy::SizeAscending
                    };

                    ui_buffers.config.contents.sort_by = sort_by;
                    ui_buffers.action = Some(UiAction::ApplySorting);
                }

                if ui
                    .selectable_label(
                        matches!(
                            app_state.config.contents.sort_by,
                            SortBy::NameAscending | SortBy::NameDescending
                        ),
                        if app_state.config.contents.sort_by == SortBy::NameDescending {
                            "🗛⏷"
                        } else {
                            "🗛⏶"
                        },
                    )
                    .on_hover_text("Sort by name")
                    .clicked()
                {
                    let sort_by = if app_state.config.contents.sort_by == SortBy::NameAscending {
                        SortBy::NameDescending
                    } else {
                        SortBy::NameAscending
                    };

                    ui_buffers.config.contents.sort_by = sort_by;
                    ui_buffers.action = Some(UiAction::ApplySorting);
                }
            });
        });
    });
}
