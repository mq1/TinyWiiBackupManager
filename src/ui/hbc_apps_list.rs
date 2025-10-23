// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(100.))
        .column(Column::auto().at_least(75.))
        .column(Column::remainder())
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.heading("🏷 Name");
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

            for hbc_app in app.filtered_hbc_apps.iter() {
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
                                app.hbc_app_info = Some(hbc_app.clone());
                            }

                            // Update button
                            if let Some(osc_apps) = &app.osc_apps
                                && let Some(osc_app) = osc_apps
                                    .iter()
                                    .find(|osc_app| osc_app.meta.name == hbc_app.meta.name)
                                && osc_app.meta.version != hbc_app.meta.version
                                && ui
                                    .button("⮉ Update")
                                    .on_hover_text(
                                        "Download update from OSC: v".to_string()
                                            + &osc_app.meta.version,
                                    )
                                    .clicked()
                            {
                                hbc_apps::spawn_install_app_from_url_task(
                                    osc_app.meta.assets.archive.url.clone(),
                                    osc_app.meta.assets.archive.size,
                                    &app.task_processor,
                                    app.config.contents.mount_point.to_path_buf(),
                                );
                            }

                            // Remove button
                            if ui
                                .button("🗑 Remove")
                                .on_hover_text("Remove HBC App")
                                .clicked()
                            {
                                app.removing_hbc_app = Some(hbc_app.clone());
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
