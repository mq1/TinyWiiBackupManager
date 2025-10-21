// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, config::WiiOutputFormat};
use eframe::egui;
use egui_theme_switch::ThemeSwitch;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("💿 Wii Output Format");

            if ui
                .radio_value(
                    &mut app.config.contents.wii_output_format,
                    WiiOutputFormat::Wbfs,
                    "WBFS (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            if ui
                .radio_value(
                    &mut app.config.contents.wii_output_format,
                    WiiOutputFormat::Iso,
                    "ISO (very large)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            ui.separator();

            ui.heading("🗐 Split Output");

            if ui
                .radio_value(
                    &mut app.config.contents.always_split,
                    false,
                    "Only when needed (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            if ui
                .radio_value(
                    &mut app.config.contents.always_split,
                    true,
                    "Always 4GB-32KB",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            ui.separator();

            ui.heading("🗑 Remove Update Partition on WBFS");

            if ui
                .radio_value(
                    &mut app.config.contents.scrub_update_partition,
                    false,
                    "No (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            if ui
                .radio_value(
                    &mut app.config.contents.scrub_update_partition,
                    true,
                    "Yes (saves some space)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            ui.separator();

            ui.heading("💣 Remove sources when adding games");

            if ui
                .radio_value(
                    &mut app.config.contents.remove_sources_games,
                    false,
                    "No (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            if ui
                .radio_value(&mut app.config.contents.remove_sources_games, true, "Yes")
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            ui.separator();

            ui.heading("💣 Remove sources when adding apps");

            if ui
                .radio_value(
                    &mut app.config.contents.remove_sources_apps,
                    false,
                    "No (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }

            if ui
                .radio_value(&mut app.config.contents.remove_sources_apps, true, "Yes")
                .changed()
                && let Err(e) = app.config.write()
            {
                app.toasts.error(e.to_string());
            }
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
            ui.add_space(5.);

            ui.horizontal(|ui| {
                if ui
                    .add(ThemeSwitch::new(&mut app.config.contents.theme_preference))
                    .changed()
                {
                    ctx.set_theme(app.config.contents.theme_preference);
                    if let Err(e) = app.config.write() {
                        app.toasts.error(e.to_string());
                    }
                }

                if ui.button("📂 Open Data Dir").clicked() {
                    if let Err(e) = open::that(&app.data_dir) {
                        app.toasts.error(e.to_string());
                    }
                }
            });
        });
    });
}
