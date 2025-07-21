// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;

/// Renders the conversion progress modal using egui::Modal
pub fn ui_conversion_modal(ctx: &egui::Context, app: &App) {
    if app.conversion_promise.is_none() {
        return;
    }

    // Create the modal dialog
    let modal = egui::Modal::new("conversion_modal".into());

    modal.show(ctx, |ui| {
        // Create a centered area for our content
        ui.vertical_centered(|ui| {
            // Title
            ui.heading("Converting ISOs");
            ui.separator();

            // Current file
            let progress = match app.conversion_progress.lock() {
                Ok(progress) => progress,
                Err(_) => {
                    ui.label("Error: Could not retrieve conversion progress.");
                    return;
                }
            };
            ui.label(&progress.current_file);
            ui.add_space(10.0);

            // Progress indicator
            if progress.is_scrubbing {
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 3.0);
                    ui.spinner();
                    ui.label("Scrubbing disc...");
                });
            } else {
                let progress_value = if progress.total_blocks > 0 {
                    progress.current_block as f32 / progress.total_blocks as f32
                } else {
                    0.0
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
