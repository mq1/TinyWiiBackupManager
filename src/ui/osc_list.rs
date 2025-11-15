// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    hbc_apps,
    ui::UiAction,
    wiiload,
};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(75.))
        .column(Column::auto().at_least(100.))
        .column(Column::remainder())
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.heading("✏ Name");
            });
            header.col(|ui| {
                ui.heading("📌 Version");
            });
            header.col(|ui| {
                ui.heading("⚖ Size");
            });
            header.col(|ui| {
                ui.heading("☞ Actions");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().spacing.item_spacing.y = 0.0;

            for osc_app_i in app_state.filtered_osc_apps.iter().copied() {
                let osc_app = &app_state.osc_apps[osc_app_i as usize];

                body.row(26., |mut row| {
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&osc_app.meta.name).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&osc_app.meta.version).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(osc_app.meta.uncompressed_size.to_string()));
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui.button("ℹ Info").on_hover_text("Show App Info").clicked() {
                                ui_buffers.action = Some(UiAction::OpenOscUrl(osc_app_i));
                            }

                            // Wiiload button
                            if ui
                                .button("📤 Wiiload")
                                .on_hover_text("Push to Wii via Wiiload")
                                .clicked()
                            {
                                let zip_url = osc_app.meta.assets.archive.url.clone();
                                let wii_ip = ui_buffers.config.contents.wii_ip.clone();
                                let task_processor = &app_state.task_processor;
                                wiiload::spawn_push_osc_task(zip_url, wii_ip, task_processor);

                                ui_buffers.action = Some(UiAction::WriteConfig);
                            }

                            // Install button
                            if ui
                                .button("📥 Install")
                                .on_hover_text("Download and Install App")
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
                        });
                        ui.separator();
                    });
                });
            }
        });
}
