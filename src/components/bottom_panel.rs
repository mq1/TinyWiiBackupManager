// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use eframe::egui;

/// Renders the bottom panel
pub fn ui_bottom_panel(ctx: &egui::Context, app: &App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            if let Some(update_info) = &app.version_check_result {
                let update_text = format!("⚠ Update available: {}", update_info.version);
                ui.hyperlink_to(update_text, &update_info.url)
                    .on_hover_text("Update to the latest version");
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "WBFS Size: {:.2} GiB",
                    app.wbfs_dir_size as f64 / 1024.0 / 1024.0 / 1024.0
                ));
                ui.label("•");
                ui.label(format!("{} games", app.games.len()));
            });
        });
    });
}
