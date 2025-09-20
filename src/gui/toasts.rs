// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::util::update_check::UpdateInfo;
use eframe::egui;
use std::time::Duration;

/// Helper function to create a styled `Toasts` instance for a specific screen corner.
pub fn create_toasts(anchor: egui_notify::Anchor) -> egui_notify::Toasts {
    egui_notify::Toasts::default()
        .with_anchor(anchor)
        .with_margin(egui::vec2(10.0, 32.0))
        .with_shadow(egui::Shadow {
            offset: [0, 0],
            blur: 0,
            spread: 1,
            color: egui::Color32::GRAY,
        })
}

pub fn show_error_toast(app: &mut App, msg: &str) {
    app.bottom_right_toasts
        .custom(
            msg,
            egui_phosphor::regular::WARNING.to_string(),
            egui::Color32::RED,
        )
        .duration(Some(Duration::from_secs(10)));
}

pub fn show_info_toast(app: &mut App, msg: &str) {
    app.bottom_right_toasts.custom(
        msg,
        egui_phosphor::regular::INFO.to_string(),
        egui::Color32::LIGHT_BLUE,
    );
}

pub fn show_update_toast(app: &mut App, update: &Option<UpdateInfo>) {
    if let Some(update) = update {
        app.bottom_left_toasts
            .custom(
                format!(
                    "{} Update available: {} {}    ",
                    egui_phosphor::regular::SPARKLE,
                    update.version,
                    egui_phosphor::regular::SPARKLE
                ),
                egui_phosphor::regular::ARROW_DOWN.to_string(),
                egui::Color32::GRAY,
            )
            .duration(Some(Duration::from_secs(10)));
    }
}

pub fn prompt_for_base_directory(app: &mut App) {
    app.top_right_toasts
        .custom(
            format!(
                "Click on \"{}\" to select a Drive/Directory {}",
                egui_phosphor::regular::LIST,
                egui_phosphor::regular::ARROW_UP
            ),
            egui_phosphor::regular::INFO.to_string(),
            egui::Color32::LIGHT_BLUE,
        )
        .closable(false)
        .duration(None);
}
