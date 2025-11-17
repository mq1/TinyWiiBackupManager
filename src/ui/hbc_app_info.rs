// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    ui::{UiAction, developers::get_developer_emoji},
};
use eframe::egui;

pub fn update(
    ctx: &egui::Context,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    hbc_app_i: u16,
) {
    let hbc_app = &app_state.hbc_apps[hbc_app_i as usize];

    egui::Modal::new("hbc_app_info".into()).show(ctx, |ui: &mut egui::Ui| {
        ui.heading(&hbc_app.meta.name);

        ui.separator();

        // Path
        ui.label(format!("📁 Path: {}", hbc_app.get_path_str()));

        ui.separator();

        ui.label(format!(
            "{} Coder: {}",
            get_developer_emoji(&hbc_app.meta.coder),
            &hbc_app.meta.coder
        ));
        ui.label("📌 Version: ".to_string() + &hbc_app.meta.version);
        ui.label("📆 Release Date: ".to_string() + &hbc_app.meta.release_date);
        ui.label("📄 Short Description: ".to_string() + &hbc_app.meta.short_description);

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(&hbc_app.meta.long_description);
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button("❌ Close").clicked() {
                ui_buffers.action = Some(UiAction::CloseModal);
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui.button("📁 Open Directory").clicked() {
                if let Err(e) = open::that(&hbc_app.path) {
                    ui_buffers.notifications.show_err(e.into());
                }
            }
        })
    });
}
