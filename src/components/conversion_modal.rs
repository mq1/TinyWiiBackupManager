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
            
            let p = &app.conversion_progress;
            ui.label(&p.current_file);
            ui.add_space(10.0);

            if p.is_scrubbing {
                ui.horizontal_centered(|ui| {
                    ui.spinner();
                    ui.label("Scrubbing disc...");
                });
            } else {
                // Calculate percentage with saturating arithmetic to avoid overflow
                // Using f32 for progress bar is fine as we only need approximate values for display
                #[allow(clippy::cast_precision_loss)]
                let percent = if p.total_blocks > 0 {
                    let percent = (p.current_block as f32 / p.total_blocks as f32).clamp(0.0, 1.0);
                    ui.add(egui::ProgressBar::new(percent).show_percentage());
                    percent
                } else {
                    ui.add(egui::ProgressBar::new(0.0).show_percentage());
                    0.0
                };

                ui.label(format!("{} / {} blocks ({:.1}%)", 
                    p.current_block, 
                    p.total_blocks, 
                    percent * 100.0
                ));
            }
        });
    });
}