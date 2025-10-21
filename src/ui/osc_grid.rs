// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;

use crate::{app::App, hbc_apps, osc::OscApp, tasks::TaskProcessor};
use eframe::egui::{self, Vec2};
use egui_notify::Toasts;

const CARD_WIDTH: f32 = 161.5;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = (available_width / (CARD_WIDTH + 20.)).floor() as usize;

        egui::Grid::new("osc")
            .num_columns(cols)
            .spacing(Vec2::splat(8.))
            .show(ui, |ui| {
                for row in app.filtered_osc_apps.chunks(cols) {
                    for osc_app in row {
                        view_osc_app_card(
                            ui,
                            osc_app,
                            &mut app.toasts,
                            &mut app.task_processor,
                            &app.config.contents.mount_point,
                        );
                    }

                    ui.end_row();
                }
            });
    });
}

fn view_osc_app_card(
    ui: &mut egui::Ui,
    osc_app: &OscApp,
    toasts: &mut Toasts,
    task_processor: &mut TaskProcessor,
    mount_point: &Path,
) {
    ui.group(|ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(&osc_app.meta.version).truncate());

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(osc_app.meta.uncompressed_size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and name
            ui.add(egui::Image::new(&osc_app.icon_uri).max_height(48.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(&osc_app.meta.name).truncate());

            ui.add_space(10.);

            // Bottom row with buttons

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Install button
                if ui
                    .button("📥")
                    .on_hover_text("Download and Install App")
                    .clicked()
                {
                    hbc_apps::spawn_install_app_from_url_task(
                        osc_app.meta.assets.archive.url.clone(),
                        osc_app.meta.assets.archive.size.clone(),
                        task_processor,
                        mount_point.to_path_buf(),
                    );
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show App Information")
                    .clicked()
                {
                    if let Err(e) = open::that(
                        "https://oscwii.org/library/app/".to_string() + &osc_app.meta.slug,
                    ) {
                        toasts.error(e.to_string());
                    }
                }
            });
        });
    });
}
