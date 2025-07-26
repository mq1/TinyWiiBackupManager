// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    if !app.conversion_in_progress {
        return;
    }

    egui::Modal::new(egui::Id::new("conversion_modal")).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Converting ISOs");
            ui.separator();
            ui.add_space(10.0);

            ui.spinner();
            ui.add_space(10.0);

            // Using f32 for progress bar is fine as we only need approximate values for display
            #[allow(clippy::cast_precision_loss)]
            let progress = if app.total_files_to_convert > 0 {
                app.files_converted as f32 / app.total_files_to_convert as f32
            } else {
                0.0
            };

            ui.add(egui::ProgressBar::new(progress).show_percentage());
            ui.add_space(5.0);

            ui.label(format!(
                "File {} of {}",
                (app.files_converted + 1).min(app.total_files_to_convert),
                app.total_files_to_convert
            ));
        });
    });
}