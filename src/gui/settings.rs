// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::settings::{ArchiveFormat, WiiOutputFormat};
use eframe::egui::{self, Rect};
use egui_theme_switch::global_theme_switch;
use strum::IntoEnumIterator;

pub fn ui_settings_window(ctx: &egui::Context, app: &mut App, rect: Rect) {
    egui::Window::new("⚙ Settings")
        .open(&mut app.settings_window_open)
        .auto_sized()
        .collapsible(false)
        .movable(false)
        .fixed_rect(rect)
        .vscroll(true)
        .show(ctx, |ui| {
            ui.add_space(10.0);

            ui.heading("📤 Wii Output Format");
            ui.add_space(10.0);

            for format in WiiOutputFormat::iter() {
                ui.radio_value(&mut app.settings.wii_output_format, format, format.as_ref());
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("🗑 Remove Update Partition on WBFS (experimental)");
            ui.add_space(10.0);

            ui.radio_value(
                &mut app.settings.remove_update_partition,
                false,
                "🛡 No (recommended)",
            );
            ui.radio_value(
                &mut app.settings.remove_update_partition,
                true,
                "💣 Yes (integrity check disabled)",
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("📦 Archive Format");
            ui.add_space(10.0);

            for format in ArchiveFormat::iter() {
                ui.radio_value(&mut app.settings.archive_format, format, format.as_ref());
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    global_theme_switch(ui);
                });
                ui.separator();
            });
        });
}
