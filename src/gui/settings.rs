// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::settings::{ArchiveFormat, WiiOutputFormat};
use eframe::egui::{self, Rect};
use egui_theme_switch::global_theme_switch;

pub fn ui_settings_window(ctx: &egui::Context, app: &mut App, rect: Rect) {
    egui::Window::new(format!("{} Settings", egui_phosphor::regular::GEAR))
        .open(&mut app.settings_window_open)
        .auto_sized()
        .collapsible(false)
        .movable(false)
        .fixed_rect(rect)
        .vscroll(true)
        .show(ctx, |ui| {
            ui.add_space(10.0);

            ui.heading(format!(
                "{} Wii Output Format",
                egui_phosphor::regular::FILE_ARCHIVE
            ));
            ui.add_space(10.0);

            ui.radio_value(
                &mut app.settings.wii_output_format,
                WiiOutputFormat::WbfsAuto,
                format!(
                    "{} {}",
                    egui_phosphor::regular::SPARKLE,
                    WiiOutputFormat::WbfsAuto
                ),
            );

            ui.radio_value(
                &mut app.settings.wii_output_format,
                WiiOutputFormat::WbfsFixed,
                format!(
                    "{} {}",
                    egui_phosphor::regular::RULER,
                    WiiOutputFormat::WbfsFixed
                ),
            );

            ui.radio_value(
                &mut app.settings.wii_output_format,
                WiiOutputFormat::Iso,
                format!("{} {}", egui_phosphor::regular::DISC, WiiOutputFormat::Iso),
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading(format!(
                "{} Remove Update Partition on WBFS (experimental)",
                egui_phosphor::regular::SCISSORS
            ));
            ui.add_space(10.0);

            ui.radio_value(
                &mut app.settings.remove_update_partition,
                false,
                format!("{} No (recommended)", egui_phosphor::regular::SHIELD),
            );
            ui.radio_value(
                &mut app.settings.remove_update_partition,
                true,
                format!(
                    "{} Yes (integrity check won't work)",
                    egui_phosphor::regular::BOMB
                ),
            );

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading(format!(
                "{} Archive Format",
                egui_phosphor::regular::PACKAGE
            ));
            ui.add_space(10.0);

            ui.radio_value(
                &mut app.settings.archive_format,
                ArchiveFormat::Rvz,
                format!("{} {}", egui_phosphor::regular::PACKAGE, ArchiveFormat::Rvz),
            );

            ui.radio_value(
                &mut app.settings.archive_format,
                ArchiveFormat::Iso,
                format!("{} {}", egui_phosphor::regular::DISC, ArchiveFormat::Iso),
            );

            ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                ui.horizontal(|ui| {
                    global_theme_switch(ui);
                });
                ui.separator();
            });
        });
}
