// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let status = app.task_processor.status.lock();

    if status.is_empty() {
        return;
    }

    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.spinner();

            let pending = app.task_processor.pending();
            if pending > 0 {
                ui.label(format!("{pending} Pending Tasks"));
            }

            ui.separator();

            ui.label(&*status);
        });
    });
}
