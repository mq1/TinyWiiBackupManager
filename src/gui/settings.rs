// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::settings::{StripPartitions, WiiOutputFormat};
use eframe::egui;
use egui_theme_switch::global_theme_switch;
use strum::IntoEnumIterator;

pub fn ui_settings_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("⚙ Settings")
        .open(&mut app.settings_window_open)
        .auto_sized()
        .collapsible(false)
        .movable(false)
        .show(ctx, |ui| {
            ui.set_width(ctx.screen_rect().width() - 14.);
            ui.set_height(ctx.screen_rect().height() - 48.);

            ui.add_space(10.0);

            ui.heading("📤 Wii Output Format");
            ui.add_space(10.0);

            for format in WiiOutputFormat::iter() {
                ui.radio_value(&mut app.settings.wii_output_format, format, format.as_ref());
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("✂ Strip Partitions on WBFS (Experimental)");
            ui.add_space(10.0);

            for format in StripPartitions::iter() {
                ui.radio_value(&mut app.settings.strip_partitions, format, format.as_ref());
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
