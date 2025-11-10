// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::{GcOutputFormat, WiiOutputFormat},
};
use eframe::egui;
use egui_theme_switch::ThemeSwitch;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing.y *= 2.;

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
                app.notifications.show_err(e);
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
                app.notifications.show_err(e);
            }

            ui.separator();

            ui.heading("💿 GameCube Output Format");

            if ui
                .radio_value(
                    &mut app.config.contents.gc_output_format,
                    GcOutputFormat::Iso,
                    "ISO (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
            }

            if ui
                .radio_value(
                    &mut app.config.contents.gc_output_format,
                    GcOutputFormat::Ciso,
                    "CISO (small but poor compatibility)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
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
                app.notifications.show_err(e);
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
                app.notifications.show_err(e);
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
                app.notifications.show_err(e);
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
                app.notifications.show_err(e);
            }

            ui.separator();

            ui.heading("💣 Delete sources when adding games");

            if ui
                .radio_value(
                    &mut app.config.contents.remove_sources_games,
                    false,
                    "No (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
            }

            if ui
                .radio_value(&mut app.config.contents.remove_sources_games, true, "Yes")
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
            }

            ui.separator();

            ui.heading("💣 Delete sources when adding apps");

            if ui
                .radio_value(
                    &mut app.config.contents.remove_sources_apps,
                    false,
                    "No (recommended)",
                )
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
            }

            if ui
                .radio_value(&mut app.config.contents.remove_sources_apps, true, "Yes")
                .changed()
                && let Err(e) = app.config.write()
            {
                app.notifications.show_err(e);
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
                        app.notifications.show_err(e);
                    }
                }
            });
        });
    });
}
