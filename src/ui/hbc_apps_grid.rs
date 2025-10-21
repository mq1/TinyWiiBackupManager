// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps::HbcApp};
use eframe::egui::{self, Vec2};

const CARD_WIDTH: f32 = 161.5;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = (available_width / (CARD_WIDTH + 20.)).floor() as usize;

        egui::Grid::new("hbc_apps")
            .num_columns(cols)
            .spacing(Vec2::splat(8.))
            .show(ui, |ui| {
                for row in app.filtered_hbc_apps.chunks(cols) {
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
}

fn view_hbc_app_card(
    ui: &mut egui::Ui,
    hbc_app: &HbcApp,
    removing_hbc_app: &mut Option<HbcApp>,
    hbc_app_info: &mut Option<HbcApp>,
) {
    ui.group(|ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(&hbc_app.meta.version).truncate());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(hbc_app.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and name
            ui.add(egui::Image::new(&hbc_app.image_uri).max_height(48.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(&hbc_app.meta.name).truncate());

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
