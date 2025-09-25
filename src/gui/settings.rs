// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::settings::{ArchiveFormat, WiiOutputFormat};
use eframe::egui;
use eframe::egui::RichText;
use egui_theme_switch::global_theme_switch;

pub fn ui_settings(ui: &mut egui::Ui, app: &mut App) {
    ui.group(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Wii Output Format",
                egui_phosphor::regular::FILE_ARCHIVE
            ))
            .strong()
            .size(14.),
        );
        ui.add_space(10.0);

        ui.radio_value(
            &mut app.settings.wii_output_format,
            WiiOutputFormat::WbfsAuto,
            format!(
                "{} WBFS Auto Split (recommended)",
                egui_phosphor::regular::SPARKLE,
            ),
        );

        ui.radio_value(
            &mut app.settings.wii_output_format,
            WiiOutputFormat::WbfsFixed,
            format!(
                "{} WBFS Fixed 4GB-32KB Split Size",
                egui_phosphor::regular::RULER,
            ),
        );

        ui.radio_value(
            &mut app.settings.wii_output_format,
            WiiOutputFormat::Iso,
            format!("{} ISO (very large)", egui_phosphor::regular::DISC),
        );

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(
            RichText::new(format!(
                "{} Remove Update Partition on WBFS (experimental)",
                egui_phosphor::regular::SCISSORS
            ))
            .strong()
            .size(14.),
        );
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

        ui.label(
            RichText::new(format!(
                "{} Archive Format",
                egui_phosphor::regular::PACKAGE
            ))
            .strong()
            .size(14.),
        );
        ui.add_space(10.0);

        ui.radio_value(
            &mut app.settings.archive_format,
            ArchiveFormat::Rvz,
            format!(
                "{} RVZ zstd-19 (recommended)",
                egui_phosphor::regular::PACKAGE
            ),
        );

        ui.radio_value(
            &mut app.settings.archive_format,
            ArchiveFormat::Iso,
            format!("{} ISO (very large)", egui_phosphor::regular::DISC),
        );

        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.horizontal(|ui| {
                global_theme_switch(ui);
            });
        });
    });
}
