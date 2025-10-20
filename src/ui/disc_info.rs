// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("disc_info".into());
    let mut close = false;

    if let Some(info) = &app.disc_info {
        modal.show(ctx, |ui| {
            ui.heading(&info.game_title);

            ui.add_space(10.);

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }
            })
        });
    }

    if close {
        app.disc_info = None;
    }
}
