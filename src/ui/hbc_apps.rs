// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::SortBy,
    hbc_apps::{self, HbcApp},
};
use eframe::egui::{self, Margin, Vec2};

const CARD_WIDTH: f32 = 153.5;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.label("Click on ☰ ⏵ 🖴 to choose a Drive or Directory");
            return;
        }

        ui.add_space(5.);
        view_top_bar(ui, app);
        ui.add_space(10.);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();
            ui.set_width(available_width);
            let cols = (available_width / (CARD_WIDTH + 28.)).floor() as usize;

            egui::Grid::new("apps")
                .num_columns(cols)
                .spacing(Vec2::splat(8.))
                .show(ui, |ui| {
                    for row in app.filtered_hbc_apps.lock().chunks(cols) {
                        for hbc_app in row {
                            view_hbc_app_card(
                                ui,
                                hbc_app,
                                &mut app.removing_hbc_app,
                                &mut app.hbc_app_info,
                            );
                        }

                        ui.end_row();
                    }
                });
        });
    });
}

fn view_top_bar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(move |ui| {
        ui.add_space(5.);
        ui.label("🔎");

        if ui
            .add(egui::TextEdit::singleline(&mut app.hbc_app_search).hint_text("Search by Name"))
            .changed()
        {
            app.update_filtered_hbc_apps();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(5.);

            if ui.button("✚ Add Apps").clicked() {}

            if ui.button("⟳").on_hover_text("Refresh Apps").clicked() {
                hbc_apps::spawn_get_hbc_apps_task(app);
            }

            ui.separator();

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
}

fn view_hbc_app_card(
    ui: &mut egui::Ui,
    hbc_app: &HbcApp,
    removing_hbc_app: &mut Option<HbcApp>,
    hbc_app_info: &mut Option<HbcApp>,
) {
    let card = egui::Frame::group(ui.style())
        .corner_radius(10.0)
        .inner_margin(Margin::same(10));

    card.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(&hbc_app.version).truncate());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(hbc_app.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and name
            ui.add(egui::Image::new(&hbc_app.image_uri).max_height(48.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(&hbc_app.name).truncate());

            ui.add_space(10.);

            // Bottom row with buttons

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Remove button
                if ui.button("🗑").on_hover_text("Remove HBC App").clicked() {
                    *removing_hbc_app = Some(hbc_app.clone());
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show App Information")
                    .clicked()
                {
                    *hbc_app_info = Some(hbc_app.clone());
                }
            });
        });
    });
}
