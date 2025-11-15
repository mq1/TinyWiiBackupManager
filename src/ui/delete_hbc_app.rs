// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    ui::UiAction,
};
use eframe::egui;

pub fn update(
    ctx: &egui::Context,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    hbc_app_i: u16,
) {
    let hbc_app = &app_state.hbc_apps[hbc_app_i as usize];

    egui::Modal::new("delete_hbc_app".into()).show(ctx, |ui| {
        ui.heading("⚠ Delete HBC App");

        ui.add_space(10.);

        ui.label(format!(
            "Are you sure you want to delete {}?",
            &hbc_app.meta.name
        ));

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Delete").clicked() {
                ui_buffers.action = Some(UiAction::DeleteHbcApp(hbc_app_i));
            }

            if ui.button("❌ Cancel").clicked() {
                ui_buffers.action = Some(UiAction::CloseModal);
            }
        });
    });
}
