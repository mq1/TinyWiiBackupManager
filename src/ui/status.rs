// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;
use egui_phosphor::regular as ph;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    if app.status.is_empty() {
        return;
    }

    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.spinner();

            let pending = app.task_processor.pending();
            if pending > 0 {
                ui.label(pending.to_string());
                ui.separator();
            }

            ui.label(&app.status);

            if pending > 0 {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{} Cancel Pending", ph::X)).clicked() {
                        app.cancel_tasks(frame);
                    }
                });
            }
        });
    });
}
