// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    hbc_apps,
    ui::{self, UiAction},
};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(100.))
        .column(Column::auto().at_least(75.))
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

            for hbc_app_i in app_state.filtered_hbc_apps.iter().copied() {
                let hbc_app = &app_state.hbc_apps[hbc_app_i as usize];

                body.row(26., |mut row| {
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&hbc_app.meta.name).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(&hbc_app.meta.version).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.add_space(3.);
                        ui.add(egui::Label::new(hbc_app.size.to_string()).truncate());
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.set_width(ui.available_width());

                        ui.horizontal(|ui| {
                            // Info button
                            if ui.button("ℹ Info").on_hover_text("Show App Info").clicked() {
                                ui_buffers.action =
                                    Some(UiAction::OpenModal(ui::Modal::HbcAppInfo(hbc_app_i)));
                            }

                            // Update button
                            if let Some(osc_app_i) = hbc_app.osc_app_i {
                                let osc_app = &app_state.osc_apps[osc_app_i as usize];

                                if osc_app.meta.version != hbc_app.meta.version
                                    && ui
                                        .button("⮉ Update")
                                        .on_hover_text(
                                            "Download update from OSC: v".to_string()
                                                + &osc_app.meta.version,
                                        )
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

                            // Delete button
                            if ui
                                .button("🗑 Delete")
                                .on_hover_text("Delete HBC App")
                                .clicked()
                            {
                                ui_buffers.action =
                                    Some(UiAction::OpenModal(ui::Modal::DeleteHbcApp(hbc_app_i)));
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
