// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::{App, ConversionState};
use crate::messages::BackgroundMessage;
use eframe::egui;
use size::Size;

pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    egui::Modal::new(egui::Id::new("conversion_modal")).show(ctx, |ui| {
        ui.set_min_width(400.0);
        ui.vertical_centered(|ui| {
            ui.heading("Converting...");
            ui.separator();
            ui.add_space(10.0);

            ui.spinner();
            ui.add_space(10.0);

            if let ConversionState::Converting {
                total_files,
                files_converted,
                current_progress: (current_progress, total),
            } = app.conversion_state
            {
                let mut progress = files_converted as f32 / total_files as f32;
                if total > 0 {
                    progress += (current_progress as f32 / total as f32) / total_files as f32;
                }

                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.add_space(5.0);

                ui.label(format!("File {} of {}", files_converted + 1, total_files));
                ui.label(format!(
                    "Progress: {} / {}",
                    Size::from_bytes(current_progress),
                    Size::from_bytes(total)
                ));

                ui.add_space(10.0);

                // Cancel button
                if ui.button("Cancel").clicked() {
                    let _ = app.inbox.sender().send(BackgroundMessage::CancelOperation);
                }
            }
        });
    });
}
