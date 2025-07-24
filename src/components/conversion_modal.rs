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
                let progress_percentage = if progress.total_blocks > 0 {
                    // Use integer arithmetic to avoid floating point precision loss warnings.
                    let current = u128::from(progress.current_block);
                    let total = u128::from(progress.total_blocks);
                    u8::try_from(current * 100 / total).unwrap_or(100)
                } else {
                    0
                };

                ui.add(
                    egui::ProgressBar::new(f32::from(progress_percentage) / 100.0)
                        .show_percentage(),
                );
                ui.label(format!(
                    "{} / {} blocks",
                    progress.current_block, progress.total_blocks
                ));
            }
        });
    });
}