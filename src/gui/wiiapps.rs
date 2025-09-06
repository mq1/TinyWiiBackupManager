// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::gui::wiiapp_grid::ui_app_grid;
use eframe::egui;

pub fn ui_apps(ui: &mut egui::Ui, app: &mut App) {
    ui_app_grid(ui, app);
}
