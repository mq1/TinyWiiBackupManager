// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use eframe::egui;

pub fn view(ctx: &egui::Context) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        ui.heading("Settings");
    });
}
