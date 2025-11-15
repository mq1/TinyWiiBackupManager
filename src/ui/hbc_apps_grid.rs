// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    hbc_apps::{self},
    ui::{self, UiAction},
};
use eframe::egui;

const CARD_WIDTH: f32 = 161.5;
const CARD_HORIZONTAL_SPACE: usize = 181;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ui: &mut egui::Ui, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = available_width as usize / CARD_HORIZONTAL_SPACE;

        let filtered_hbc_apps = &app_state.filtered_hbc_apps;

        ui.heading(format!(
            "★ Homebrew Channel Apps: {} found ({})",
            filtered_hbc_apps.len(),
            &app_state.filtered_hbc_apps_size
        ));

        ui.add_space(5.);

        for row in filtered_hbc_apps.chunks(cols) {
            ui.horizontal_top(|ui| {
                for hbc_app_i in row.iter().copied() {
                    update_hbc_app_card(ui, app_state, ui_buffers, hbc_app_i);
                }
            });

            ui.add_space(5.);
        }
    });
}

fn update_hbc_app_card(
    ui: &mut egui::Ui,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    hbc_app_i: u16,
) {
    let hbc_app = &app_state.hbc_apps[hbc_app_i as usize];

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
                    ui_buffers.action =
                        Some(UiAction::OpenModal(ui::Modal::DeleteHbcApp(hbc_app_i)));
                }

                // Update button
                if let Some(osc_app_i) = hbc_app.osc_app_i {
                    let osc_app = &app_state.osc_apps[osc_app_i as usize];

                    if osc_app.meta.version != hbc_app.meta.version
                        && ui
                            .button("⮉")
                            .on_hover_text(format!(
                                "Download update from OSC: v{}",
                                &osc_app.meta.version
                            ))
                            .clicked()
                    {
                        let zip_url = osc_app.meta.assets.archive.url.clone();
                        let task_processor = &app_state.task_processor;
                        let mount_point = app_state.config.contents.mount_point.clone();

                        hbc_apps::spawn_install_app_from_url_task(
                            zip_url,
                            task_processor,
                            mount_point,
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
                    ui_buffers.action = Some(UiAction::OpenModal(ui::Modal::HbcAppInfo(hbc_app_i)));
                }
            });
        });
    });
}
