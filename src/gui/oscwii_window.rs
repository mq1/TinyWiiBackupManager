// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;
use eframe::egui::RichText;

pub fn ui_oscwii_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("📥 Open Shop Channel")
        .open(&mut app.oscwii_window_open)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter");
                ui.text_edit_singleline(&mut app.oscwii_filter);
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                egui::Grid::new("oscwii_apps")
                    .striped(true)
                    .start_row(1)
                    .show(ui, |ui| {
                        ui.label(RichText::new("App").strong());
                        ui.label(RichText::new("📥 Download").strong());
                        ui.label(RichText::new("📮 Wiiload").strong());
                        ui.end_row();

                        for wiiapp in app.oscwii_apps.apps.iter().filter(|wiiapp| {
                            wiiapp
                                .name
                                .to_lowercase()
                                .contains(&app.oscwii_filter.to_lowercase())
                        }) {
                            ui.hyperlink_to(
                                &wiiapp.name,
                                format!("https://oscwii.org/library/app/{}", wiiapp.slug),
                            );
                            if app.base_dir.is_some() {
                                let _ = ui.button(format!("⬇ {}", wiiapp.version));
                            }
                            let _ = ui.button(format!("⬆ {}", wiiapp.version));
                            ui.end_row();
                        }
                    });
            });
        });
}
