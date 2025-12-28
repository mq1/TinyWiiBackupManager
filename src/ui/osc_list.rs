// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{hbc_apps, ui, wiiload};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular as ph;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    TableBuilder::new(ui)
        .resizable(true)
        .column(Column::auto().at_least(250.))
        .column(Column::auto().at_least(75.))
        .column(Column::auto().at_least(100.))
        .column(Column::remainder())
        .header(26.0, |mut header| {
            header.col(|ui| {
                ui.heading("Name");
            });
            header.col(|ui| {
                ui.heading("Version");
            });
            header.col(|ui| {
                ui.heading("Size");
            });
            header.col(|ui| {
                ui.heading("Actions");
            });
        })
        .body(|mut body| {
            body.ui_mut().style_mut().spacing.item_spacing.y = 0.0;

            for osc_app_i in app.filtered_osc_apps.iter().copied() {
                let osc_app = &app.osc_apps[osc_app_i as usize];

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
                        ui.add(egui::Label::new(&osc_app.meta.uncompressed_size));
                        ui.add_space(3.);
                        ui.separator();
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            // Info button
                            if ui
                                .button(format!("{} Info", ph::INFO))
                                .on_hover_text("Show App Info")
                                .clicked()
                            {
                                app.send_msg(Message::OpenModal(ui::Modal::OscAppInfo(osc_app_i)));
                            }

                            // Wiiload button
                            if ui
                                .button(format!("{} Wiiload", ph::MONITOR_ARROW_UP))
                                .on_hover_text("Push to Wii via Wiiload")
                                .clicked()
                            {
                                wiiload::spawn_push_osc_task(
                                    app,
                                    osc_app.meta.assets.archive.url.clone(),
                                );

                                app.send_msg(Message::WriteConfig);
                            }

                            // Install button
                            if ui
                                .button(format!("{} Install", ph::CLOUD_ARROW_DOWN))
                                .on_hover_text("Download and Install App")
                                .clicked()
                            {
                                hbc_apps::spawn_install_app_from_url_task(
                                    app,
                                    osc_app.meta.assets.archive.url.clone(),
                                );
                            }
                        });
                        ui.separator();
                    });
                });
            }
        });
}
