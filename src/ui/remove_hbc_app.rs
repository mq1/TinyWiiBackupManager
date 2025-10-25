// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;
use std::fs;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("remove_hbc_app".into());
    let mut close = false;
    let mut refresh = false;

    if let Some(hbc_app) = &app.removing_hbc_app {
        let text = format!("Are you sure you want to remove {}?", &hbc_app.meta.name);

        modal.show(ctx, |ui| {
            ui.heading("⚠ Remove HBC App");

            ui.add_space(10.);

            ui.label(text);

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑 Remove").clicked() {
                    if let Err(e) = fs::remove_dir_all(&hbc_app.path) {
                        app.toasts.error(e.to_string());
                    }

                    close = true;
                    refresh = true;
                }

                if ui.button("❌ Cancel").clicked() {
                    close = true;
                }
            });
        });
    }

    if close {
        app.removing_hbc_app = None;
    }

    if refresh {
        app.refresh_hbc_apps(ctx);
    }
}
