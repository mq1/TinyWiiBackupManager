// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

pub fn ui_oscwii_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("📥 Open Shop Channel")
        .open(&mut app.oscwii_window_open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter");
                ui.text_edit_singleline(&mut app.oscwii_filter);
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for wiiapp in app.oscwii_apps.apps.iter().filter(|wiiapp| {
                    wiiapp
                        .name
                        .to_lowercase()
                        .contains(&app.oscwii_filter.to_lowercase())
                }) {
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label(&wiiapp.name);
                        ui.label(&wiiapp.version);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(15.);
                            ui.button("⬇")
                        });
                    });
                }
            });
        });
}
