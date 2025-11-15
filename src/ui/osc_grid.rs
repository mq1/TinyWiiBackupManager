// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    hbc_apps,
    ui::UiAction,
    wiiload,
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

        ui.heading(format!(
            "🏪 Open Shop Channel Apps: {} found",
            app_state.filtered_osc_apps.len(),
        ));

        ui.add_space(5.);

        for row in app_state.filtered_osc_apps.chunks(cols) {
            ui.horizontal_top(|ui| {
                for osc_app_i in row.iter().copied() {
                    update_osc_app_card(ui, app_state, ui_buffers, osc_app_i);
                }
            });

            ui.add_space(5.);
        }
    });
}

fn update_osc_app_card(
    ui: &mut egui::Ui,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    i: u16,
) {
    let osc_app = &app_state.osc_apps[i as usize];

    let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.label(osc_app.meta.version_display());

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
                    let zip_url = osc_app.meta.assets.archive.url.clone();
                    let task_processor = &app_state.task_processor;
                    let mount_point = app_state.config.contents.mount_point.clone();
                    hbc_apps::spawn_install_app_from_url_task(zip_url, task_processor, mount_point);
                }

                // Wiiload button
                if ui
                    .button("📤")
                    .on_hover_text("Push to Wii via Wiiload")
                    .clicked()
                {
                    let zip_url = osc_app.meta.assets.archive.url.clone();
                    let wii_ip = ui_buffers.config.contents.wii_ip.clone();
                    let task_processor = &app_state.task_processor;
                    wiiload::spawn_push_osc_task(zip_url, wii_ip, task_processor);

                    ui_buffers.action = Some(UiAction::WriteConfig);
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show App Information")
                    .clicked()
                {
                    ui_buffers.action = Some(UiAction::OpenOscUrl(i));
                }
            });
        });
    });
}
