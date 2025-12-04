// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App, hbc_app_i: u16) {
    egui::Modal::new("delete_hbc_app".into()).show(ctx, |ui| {
        ui.heading(format!(
            "{} Delete HBC App",
            egui_phosphor::regular::WARNING
        ));

        ui.add_space(10.);

        ui.label(format!(
            "Are you sure you want to delete {}?",
            &app.hbc_apps[hbc_app_i as usize].meta.name
        ));

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .button(format!("{} Delete", egui_phosphor::regular::TRASH))
                .clicked()
            {
                app.delete_hbc_app(ctx, hbc_app_i);
                app.close_modal();
            }

            if ui
                .button(format!("{} Cancel", egui_phosphor::regular::X))
                .clicked()
            {
                app.close_modal();
            }
        });
    });
}
