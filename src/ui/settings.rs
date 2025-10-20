// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, config::WiiOutputFormat};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("💿 Wii Output Format");

            if ui
                .radio_value(
                    &mut app.config.contents.wii_output_format,
                    WiiOutputFormat::Wbfs,
                    "WBFS (Recommended)",
                )
                .changed()
            {
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
            }

            if ui
                .radio_value(
                    &mut app.config.contents.wii_output_format,
                    WiiOutputFormat::Iso,
                    "ISO (very large)",
                )
                .changed()
            {
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
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
            {
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
            }

            if ui
                .radio_value(&mut app.config.contents.always_split, true, "Always")
                .changed()
            {
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
            }
        });
    });
}
