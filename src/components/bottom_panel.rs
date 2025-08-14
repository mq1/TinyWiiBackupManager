// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

/// Renders the bottom panel
pub fn ui_bottom_panel(ctx: &egui::Context, app: &App) {
    if let Some(update_info) = &app.version_check_result {
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            let update_text = format!("⚠ Update available: {}", update_info.version);
            ui.hyperlink_to(update_text, &update_info.url)
                .on_hover_text("Update to the latest version");
        });
    }
}
