// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    config::{GcOutputFormat, WiiOutputFormat},
    ui::UiAction,
};
use eframe::egui;
use egui_theme_switch::ThemeSwitch;

pub fn update(ctx: &egui::Context, _app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing.y *= 2.;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("💿 Wii Output Format");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.wii_output_format,
                    WiiOutputFormat::Wbfs,
                    "WBFS (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.wii_output_format,
                    WiiOutputFormat::Iso,
                    "ISO (very large)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            ui.separator();

            ui.heading("💿 GameCube Output Format");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.gc_output_format,
                    GcOutputFormat::Iso,
                    "ISO (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.gc_output_format,
                    GcOutputFormat::Ciso,
                    "CISO (small but poor compatibility)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            ui.separator();

            ui.heading("🗐 Split Output");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.always_split,
                    false,
                    "Only when needed (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.always_split,
                    true,
                    "Always 4GB-32KB",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            ui.separator();

            ui.heading("🗑 Remove Update Partition on WBFS");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.scrub_update_partition,
                    false,
                    "No (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.scrub_update_partition,
                    true,
                    "Yes (saves some space)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            ui.separator();

            ui.heading("💣 Delete sources when adding games");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.remove_sources_games,
                    false,
                    "No (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.remove_sources_games,
                    true,
                    "Yes",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            ui.separator();

            ui.heading("💣 Delete sources when adding apps");

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.remove_sources_apps,
                    false,
                    "No (recommended)",
                )
                .changed()
            {
                ui_buffers.save_config();
            }

            if ui
                .radio_value(
                    &mut ui_buffers.config.contents.remove_sources_apps,
                    true,
                    "Yes",
                )
                .changed()
            {
                ui_buffers.save_config();
            }
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.add_space(5.);

            ui.horizontal(|ui| {
                if ui
                    .add(ThemeSwitch::new(
                        &mut ui_buffers.config.contents.theme_preference,
                    ))
                    .changed()
                {
                    ctx.set_theme(ui_buffers.config.contents.theme_preference);
                    ui_buffers.save_config();
                }
            });
        });
    });
}
