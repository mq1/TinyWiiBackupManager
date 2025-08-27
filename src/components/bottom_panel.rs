// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, components::console_filter::ui_console_filter};
use eframe::egui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// --- UI Rendering ---

/// Renders the bottom panel, which includes the update notifier and other controls.
pub fn ui_bottom_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if let Some(update_info) = &app.update_info {
                ui.hyperlink_to(format!("{} (new)", &update_info.version), &update_info.url);
            } else {
                ui.label(format!("v{}", VERSION));
            }

            // Layout for other controls, aligned to the right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut app.remove_sources, "💣 Remove sources")
                    .on_hover_text("⚠ DANGER ⚠\n\nThis will delete the input files!");

                ui.separator();
                ui_console_filter(ui, &mut app.console_filter);

                // Show active jobs count
                let active_count = app.jobs.active_count();
                if active_count > 0 {
                    ui.separator();

                    // Show job count with progress
                    let job_text = format!(
                        "⚙ {} job{}",
                        active_count,
                        if active_count == 1 { "" } else { "s" }
                    );

                    // Show first job's progress as tooltip
                    if let Some(job) = app.jobs.jobs.first() {
                        if let Ok(status) = job.context.status.read() {
                            let mut tooltip = status.status.clone();
                            if let Some([current, total]) = status.progress_items {
                                tooltip.push_str(&format!(" ({}/{})", current, total));
                            }
                            ui.label(job_text).on_hover_text(tooltip);
                        } else {
                            ui.label(job_text);
                        }
                    } else {
                        ui.label(job_text);
                    }
                }
            });
        });
    });
}
