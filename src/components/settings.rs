// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::settings::OutputFormat;
use eframe::egui;
use egui_theme_switch::global_theme_switch;
use strum::IntoEnumIterator;

pub fn ui_settings_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("⚙ Settings")
        .open(&mut app.settings_window_open)
        .show(ctx, |ui| {
            ui.add_space(10.0);

            ui.heading("📤 Output Format");
            ui.add_space(10.0);

            for format in OutputFormat::iter() {
                ui.radio_value(&mut app.settings.output_format, format, format.as_ref());
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("🎨 Theme");
                global_theme_switch(ui);
            });
        });
}
