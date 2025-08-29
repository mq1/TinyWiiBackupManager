// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, components::console_filter::ui_console_filter};
use eframe::egui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Renders the bottom panel, which includes the update notifier and other controls.
pub fn ui_bottom_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if app.task_processor.is_idle() && app.status.is_empty() {
                // Show version and update notifier if app is idle
                if let Some(update_info) = &app.update_info {
                    ui.hyperlink_to(format!("{} (new)", &update_info.version), &update_info.url);
                } else {
                    ui.label(format!("v{}", VERSION));
                }
            } else {
                // Show spinner if there are running tasks
                ui.spinner();
            }

            // Show status
            ui.label(&app.status);

            // Layout for other controls, aligned to the right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut app.remove_sources, "💣 Remove sources")
                    .on_hover_text("⚠ DANGER ⚠\n\nThis will delete the input files!");

                ui.separator();
                ui_console_filter(ui, &mut app.console_filter);
            });
        });
    });
}
