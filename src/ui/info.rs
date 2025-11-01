// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("info".into());

    if app.is_info_open {
        modal.show(ctx, |ui: &mut egui::Ui| {
            ui.horizontal(|ui| {
                ui.set_height(68.);

                ui.add(
                    egui::Image::from_bytes("bytes://info", ui::LOGO_BYTES)
                        .max_size(egui::Vec2::splat(64.)),
                );

                ui.vertical(|ui| {
                    ui.add_space(4.);
                    ui.heading(env!("CARGO_PKG_NAME"));
                    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                    ui.label("© 2025 Manuel Quarneti");
                });
            });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("❌ Close").clicked() {
                    app.is_info_open = false;
                }

                if ui.button("📁 Open Data Directory").clicked()
                    && let Err(e) = open::that(&app.data_dir)
                {
                    app.toasts.error(e.to_string());
                }
            })
        });
    }
}
