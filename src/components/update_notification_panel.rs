// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;

/// Renders the update notification panel at the bottom
pub fn ui_update_notification_panel(ctx: &egui::Context, app: &App) {
    if let Some(update_info) = &app.version_check_result {
        egui::TopBottomPanel::bottom("update_panel").show(ctx, |ui| {
            if ui
                .button(format!(
                    "⚠ Update available: {}",
                    update_info.version
                ))
                .clicked()
            {
                if let Err(e) = webbrowser::open(&update_info.url) {
                    log::error!("Failed to open browser: {}", e);
                }
            }
        });
    }
}
