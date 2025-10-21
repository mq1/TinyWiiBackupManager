// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, hbc_apps};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::remainder())
        .columns(Column::auto(), 3)
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("🏷 Name");
            });
            header.col(|ui| {
                ui.heading("📌 Version    ");
            });
            header.col(|ui| {
                ui.heading("⚖ Size    ");
            });
            header.col(|ui| {
                ui.heading("☞ Actions");
            });
        })
        .body(|mut body| {
            for osc_app in &app.filtered_osc_apps {
                body.row(20., |mut row| {
                    row.col(|ui| {
                        ui.label(&osc_app.meta.name);
                    });
                    row.col(|ui| {
                        ui.label(&osc_app.meta.version);
                    });
                    row.col(|ui| {
                        ui.label(osc_app.meta.uncompressed_size.to_string());
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Install button
                            if ui
                                .button("📥")
                                .on_hover_text("Download and Install App")
                                .clicked()
                            {
                                hbc_apps::spawn_install_app_from_url_task(
                                    osc_app.meta.assets.archive.url.clone(),
                                    osc_app.meta.assets.archive.size.clone(),
                                    &app.task_processor,
                                    app.config.contents.mount_point.clone(),
                                );
                            }

                            // Info button
                            if ui.button("ℹ").on_hover_text("Show App Info").clicked() {
                                if let Err(e) = open::that(
                                    "https://oscwii.org/library/app/".to_string()
                                        + &osc_app.meta.slug,
                                ) {
                                    app.toasts.error(e.to_string());
                                }
                            }
                        });
                    });
                });
            }
        });
}
