// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

/// Renders the conversion progress modal.
pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    // Early return if no conversion is in progress
    let Some(_) = &app.conversion_promise else { return };

    // Create and show the modal dialog
    egui::Modal::new("conversion_modal".into()).show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Converting ISOs");
            ui.separator();

            // Safely acquire the progress lock
            let Ok(progress) = app.conversion_progress.lock() else {
                ui.label("Error: Could not retrieve conversion progress.");
                return;
            };

            // Display the current file being processed
            ui.label(&progress.current_file);
            ui.add_space(10.0);

            // Show different UI based on the conversion stage
            if progress.is_scrubbing {
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 3.0); // Center the spinner
                    ui.spinner();
                    ui.label("Scrubbing disc...");
                });
            } else {
                // Calculate and display conversion progress
                let progress_value = if progress.total_blocks > 0 {
                    progress.current_block as f32 / progress.total_blocks as f32
                } else {
                    0.0 // Avoid division by zero
                };
                ui.add(egui::ProgressBar::new(progress_value).show_percentage());
                ui.label(format!(
                    "{} / {} blocks",
                    progress.current_block, progress.total_blocks
                ));
            }
        });
    });
}