// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
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
            for hbc_app in app.filtered_hbc_apps.iter() {
                body.row(20., |mut row| {
                    row.col(|ui| {
                        ui.label(&hbc_app.meta.name);
                    });
                    row.col(|ui| {
                        ui.label(&hbc_app.meta.version);
                    });
                    row.col(|ui| {
                        ui.label(hbc_app.size.to_string());
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui.button("ℹ").on_hover_text("Show App Info").clicked() {
                                app.hbc_app_info = Some(hbc_app.clone());
                            }
                            // Remove button
                            if ui.button("🗑").on_hover_text("Remove HBC App").clicked() {
                                app.removing_hbc_app = Some(hbc_app.clone());
                            }
                        });
                    });
                });
            }
        });
}
