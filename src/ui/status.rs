// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::{AppState, UiBuffers};
use eframe::egui;

pub fn update(ctx: &egui::Context, app_state: &AppState, _ui_buffers: &mut UiBuffers) {
    if app_state.status.is_empty() {
        return;
    }

    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            ui.spinner();

            let pending = app_state.task_processor.pending();
            if pending > 0 {
                ui.label(format!("{pending} Pending Tasks"));
            }

            ui.separator();

            ui.label(&app_state.status);
        });
    });
}
