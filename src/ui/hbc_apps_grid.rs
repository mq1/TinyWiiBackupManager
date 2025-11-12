// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    hbc_apps::{self, HbcApp},
    osc::OscApp,
    tasks::TaskProcessor,
    ui,
};
use eframe::egui;
use std::path::Path;

const CARD_WIDTH: f32 = 161.5;
const CARD_HORIZONTAL_SPACE: usize = 181;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = available_width as usize / CARD_HORIZONTAL_SPACE;

        ui.heading(format!(
            "★ Homebrew Channel Apps: {} found ({})",
            app.filtered_hbc_apps.len(),
            app.filtered_hbc_apps_size
        ));

        ui.add_space(5.);

        for row in app.filtered_hbc_apps.chunks(cols) {
            ui.horizontal_top(|ui| {
                for hbc_app_i in row {
                    view_hbc_app_card(
                        ui,
                        hbc_app_i,
                        &app.hbc_apps[*hbc_app_i as usize],
                        &mut app.current_modal,
                        &app.osc_apps,
                        &app.task_processor,
                        &app.config.contents.mount_point,
                    );
                }
            });

            ui.add_space(5.);
        }
    });
}

fn view_hbc_app_card(
    ui: &mut egui::Ui,
    hbc_app_i: &u16,
    hbc_app: &HbcApp,
    current_modal: &mut ui::Modal,
    osc_apps: &[OscApp],
    task_processor: &TaskProcessor,
    mount_point: &Path,
) {
    let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.label(hbc_app.meta.version_display());

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
                // Delete button
                if ui.button("🗑").on_hover_text("Delete HBC App").clicked() {
                    *current_modal = ui::Modal::DeleteHbcApp(*hbc_app_i);
                }

                // Update button
                if let Some(osc_app_i) = hbc_app.osc_app_i {
                    let osc_app = &osc_apps[osc_app_i];

                    if osc_app.meta.version != hbc_app.meta.version
                        && ui
                            .button("⮉")
                            .on_hover_text(
                                "Download update from OSC: v".to_string() + &osc_app.meta.version,
                            )
                            .clicked()
                    {
                        hbc_apps::spawn_install_app_from_url_task(
                            osc_app.meta.assets.archive.url.clone(),
                            task_processor,
                            mount_point.to_path_buf(),
                        );
                    }
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show App Information")
                    .clicked()
                {
                    *current_modal = ui::Modal::HbcAppInfo(*hbc_app_i);
                }
            });
        });
    });
}
