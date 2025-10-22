// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps, wiiload};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(75.))
        .column(Column::auto().at_least(100.))
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

            for osc_app in &app.filtered_osc_apps {
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
                        ui.add(egui::Label::new(
                            &osc_app.meta.uncompressed_size.to_string(),
                        ));
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui.button("ℹ Info").on_hover_text("Show App Info").clicked()
                                && let Err(e) = open::that(
                                    "https://oscwii.org/library/app/".to_string()
                                        + &osc_app.meta.slug,
                                )
                            {
                                app.toasts.error(e.to_string());
                            }

                            // Wiiload button
                            if ui
                                .button("📤 Wiiload")
                                .on_hover_text("Push to Wii via Wiiload")
                                .clicked()
                            {
                                if let Err(e) = app.config.write() {
                                    app.toasts.error(e.to_string());
                                }

                                wiiload::spawn_push_osc_task(
                                    osc_app.meta.assets.archive.url.clone(),
                                    osc_app.meta.assets.archive.size,
                                    app.config.contents.wii_ip.clone(),
                                    &app.task_processor,
                                );
                            }

                            // Install button
                            if ui
                                .button("📥 Install")
                                .on_hover_text("Download and Install App")
                                .clicked()
                            {
                                hbc_apps::spawn_install_app_from_url_task(
                                    osc_app.meta.assets.archive.url.clone(),
                                    osc_app.meta.assets.archive.size,
                                    &app.task_processor,
                                    app.config.contents.mount_point.clone(),
                                );
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
