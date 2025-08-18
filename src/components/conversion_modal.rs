// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, ConversionState};
use eframe::egui;

pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    egui::Modal::new(egui::Id::new("conversion_modal")).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Converting ISOs");
            ui.separator();
            ui.add_space(10.0);

            ui.spinner();
            ui.add_space(10.0);

            if let ConversionState::Converting { total_files, files_converted } = app.conversion_state {
                let progress = files_converted as f32 / total_files as f32;

                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.add_space(5.0);

                ui.label(format!(
                    "File {} of {}",
                    files_converted + 1,
                    total_files
                ));
            }
        });
    });
}