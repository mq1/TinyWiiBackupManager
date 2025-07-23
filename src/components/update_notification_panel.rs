// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, error_handling};
use eframe::egui;

/// Renders the update notification panel at the bottom.
pub fn ui_update_notification_panel(ctx: &egui::Context, app: &App) {
    // Only show the panel if an update is available
    if let Some(update_info) = &app.version_check_result {
        egui::TopBottomPanel::bottom("update_panel").show(ctx, |ui| {
            // Button to notify about the available update
            if ui
                .button(format!("⚠ Update available: {}", update_info.version))
                .clicked()
            {
                // Attempt to open the update URL in the default web browser
                if let Err(e) = webbrowser::open(&update_info.url) {
                    error_handling::show_error(
                        "Error opening browser",
                        &format!("Failed to open browser: {e}"),
                    );
                }
            }
        });
    }
}