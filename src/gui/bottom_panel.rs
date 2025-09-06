// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::View;
use crate::{app::App, gui::console_filter::ui_console_filter};
use eframe::egui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Renders the bottom panel, which includes the update notifier and other controls.
pub fn ui_bottom_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // If the app is busy, show the number of tasks in queue and a spinner
            if let Some(status) = &app.task_status {
                ui.spinner();

                // show number of tasks
                let queued = app.task_processor.tasks_count();
                ui.label(format!(
                    "{} task{}",
                    queued + 1,
                    if queued > 0 { "s" } else { "" }
                ));

                ui.separator();

                // Label::new(status).truncate() does not truncate the text for some reason
                let truncated = status.chars().take(50).collect::<String>();
                let dots = if status.len() > 50 { "..." } else { "" };
                ui.label(format!("{truncated}{dots}"));
            }
            // If the app is idle, show the update notifier and version
            else {
                if let Some(update_info) = &app.update_info {
                    ui.hyperlink_to(format!("{} (new)", &update_info.version), &update_info.url);
                } else {
                    ui.label(format!("v{}", VERSION));
                }
            }

            // Layout for other controls, aligned to the right.
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| match app.view {
                    View::Games => {
                        ui_console_filter(ui, &mut app.console_filter);
                        ui.separator();
                        ui.checkbox(&mut app.remove_sources, "💣 Remove sources")
                            .on_hover_text("⚠ DANGER ⚠\n\nThis will delete the input files!");
                    }
                    View::WiiApps => {}
                },
            );
        });
    });
}
