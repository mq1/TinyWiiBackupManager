// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{banners, covers, txtcodes, wiitdb};
use eframe::egui;
use egui_phosphor::fill as ph;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading(format!(
                "Click on {} to select a Drive/Mount Point",
                ph::HARD_DRIVE
            ));
            return;
        }

        ui.style_mut().spacing.item_spacing.y *= 2.;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let style = ui.style();
            let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!("{} USB Loader GX", ph::MAGIC_WAND));

                ui.horizontal(|ui| {
                    if ui.button(ph::CLOUD_ARROW_DOWN).clicked() {
                        wiitdb::spawn_download_task(app);
                        wiitdb::spawn_load_wiitdb_task(app);
                    }

                    ui.label("Download wiitdb.xml (overwrites existing one)");
                });

                ui.horizontal(|ui| {
                    if ui.button(ph::CLOUD_ARROW_DOWN).clicked() {
                        covers::spawn_download_all_covers_task(app);
                    }

                    ui.label("Download all covers (defaults to English for PAL games, while usbloader_gx downloads them in the correct language)");
                });

                ui.horizontal(|ui| {
                    if ui.button(ph::CLOUD_ARROW_DOWN).clicked() {
                        banners::spawn_download_banners_task(app);
                    }

                    ui.label("Download banners (GameCube only)");
                });
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!("{} WiiFlow Lite", ph::SHUFFLE));

                ui.horizontal(|ui| {
                    if ui.button(ph::CLOUD_ARROW_DOWN).clicked() {
                        covers::spawn_download_wiiflow_covers_task(app);
                    }

                    ui.label("Download all covers (defaults to English for PAL games)");
                });
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!("{} Cheat Codes", ph::FILE_TXT));

                ui.horizontal(|ui| {
                    if ui.button(ph::CLOUD_ARROW_DOWN).clicked() {
                        txtcodes::spawn_download_all_cheats_task(app);
                    }

                    ui.label("Download cheats for all games (txt)");
                });
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!("{} Cleanup", ph::BROOM));

                ui.horizontal(|ui| {
                    if ui.button(ph::ARROW_FAT_RIGHT).clicked() {
                        app.run_normalize_paths();
                    }

                    ui.label("Normalize paths (makes sure the game directories' layouts are correct)");
                });

                ui.horizontal(|ui| {
                    if ui.button(ph::ARROW_FAT_RIGHT).clicked() {
                        app.run_strip_all_games(frame);
                    }

                    ui.label("Remove the update partition from all .wbfs files");
                });
            });

            if cfg!(target_os = "macos") {
                group.show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    ui.heading(format!("{} macOS", ph::APPLE_LOGO));

                    ui.horizontal(|ui| {
                        if ui.button(ph::ARROW_FAT_RIGHT).clicked() {
                            app.run_dot_clean();
                        }

                        ui.label("Run dot_clean (remove hidden ._ files)");
                    });
                });
            }
        });
    });
}
