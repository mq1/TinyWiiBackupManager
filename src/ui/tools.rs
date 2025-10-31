// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, banners, covers, txtcodes, util, wiitdb};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on 🖴 to choose a Drive or Directory");
            return;
        }

        ui.style_mut().spacing.item_spacing.y *= 2.;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("🔧 USB Loader GX");

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    wiitdb::spawn_download_task(app);
                }

                ui.label("Download wiitdb.xml");
            });

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    covers::spawn_download_all_covers_task(app);
                }

                ui.label("Download all covers");
            });

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    banners::spawn_download_banners_task(app);
                }

                ui.label("Download banners (GameCube only)");
            });

            ui.separator();
            ui.heading("🛠 Cheat Codes");

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    txtcodes::spawn_download_cheats_task(app);
                }

                ui.label("Download cheats for all games (txt)");
            });

            if cfg!(target_os = "macos") {
                ui.separator();
                ui.heading(" macOS");

                ui.horizontal(|ui| {
                    if ui.button("⏵").clicked() {
                        match util::run_dot_clean(&app.config.contents.mount_point) {
                            Ok(()) => app.toasts.success("dot_clean successful"),
                            Err(err) => app.toasts.error(err.to_string()),
                        };
                    }

                    ui.label("Run dot_clean (remove hidden ._ files)");
                });
            }
        });
    });
}
