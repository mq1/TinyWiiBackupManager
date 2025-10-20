// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("hbc_app_info".into());
    let mut close = false;

    if let Some(info) = &app.hbc_app_info {
        modal.show(ctx, |ui: &mut egui::Ui| {
            ui.heading(&info.meta.name);

            ui.separator();

            // Path
            ui.label("📁 Path: ".to_string() + info.path.to_str().unwrap_or("Unknown"));

            ui.separator();

            ui.label("Coder: ".to_string() + &info.meta.coder);
            ui.label("Version: ".to_string() + &info.meta.version);
            ui.label("Release Date: ".to_string() + &info.meta.release_date);
            ui.label("Short Description: ".to_string() + &info.meta.short_description);

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.label(&info.meta.long_description);
                });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }

                if ui.button("📁 Open Directory").clicked() {
                    if let Err(e) = open::that(&info.path) {
                        app.toasts.error(e.to_string());
                    }
                }
            })
        });
    }

    if close {
        app.hbc_app_info = None;
    }
}
