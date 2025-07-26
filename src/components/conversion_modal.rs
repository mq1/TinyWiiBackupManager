// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    egui::Modal::new(egui::Id::new("conversion_modal")).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Converting ISOs");
            ui.separator();
            ui.add_space(10.0);

            ui.spinner();
            ui.add_space(10.0);

            let progress = app.files_converted / app.total_files_to_convert;

            ui.add(egui::ProgressBar::new(progress as f32).show_percentage());
            ui.add_space(5.0);

            ui.label(format!(
                "File {} of {}",
                (app.files_converted + 1).min(app.total_files_to_convert),
                app.total_files_to_convert
            ));
        });
    });
}