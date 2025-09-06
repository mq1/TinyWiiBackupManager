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
        .error(msg)
        .duration(Some(Duration::from_secs(10)));
}

pub fn show_info_toast(app: &mut App, msg: &str) {
    app.bottom_right_toasts.info(msg);
}

pub fn show_update_toast(app: &mut App, update: &Option<UpdateInfo>) {
    if let Some(update) = update {
        app.bottom_left_toasts
            .custom(
                format!("✨Update available: {}✨    ", update.version),
                "⬇".to_string(),
                egui::Color32::GRAY,
            )
            .duration(Some(Duration::from_secs(10)));
    }
}

pub fn prompt_for_base_directory(app: &mut App) {
    app.top_right_toasts
        .info("Click on \"☰\" to select a Drive/Directory  ⬆")
        .closable(false)
        .duration(None);
}
