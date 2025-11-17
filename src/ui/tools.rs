// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    banners, covers, txtcodes,
    ui::UiAction,
    wiitdb,
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if ui_buffers.config.contents.mount_point.as_os_str().is_empty() {
            ui.heading("Click on 🖴 to select a Drive/Mount Point");
            return;
        }

        ui.style_mut().spacing.item_spacing.y *= 2.;

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("💫 USB Loader GX");

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    wiitdb::spawn_download_task(&app_state.task_processor, ui_buffers.config.contents.mount_point.clone());
                    wiitdb::spawn_load_wiitdb_task(&app_state.task_processor, ui_buffers.config.contents.mount_point.clone());
                }

                ui.label("Download wiitdb.xml (overwrites existing one)");
            });

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    covers::spawn_download_all_covers_task(&app_state.task_processor, ui_buffers.config.contents.mount_point.clone(), app_state.games.clone().into_boxed_slice());
                }

                ui.label("Download all covers (defaults to English for PAL games, while usbloader_gx downloads them in the correct language)");
            });

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    banners::spawn_download_banners_task(&app_state.task_processor, &app_state.games,  &ui_buffers.config.contents.mount_point);
                }

                ui.label("Download banners (GameCube only)");
            });

            ui.separator();
            ui.heading("🔀 WiiFlow Lite");

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    covers::spawn_download_wiiflow_covers_task(&app_state.task_processor, ui_buffers.config.contents.mount_point.clone(), app_state.games.clone().into_boxed_slice());
                }

                ui.label("Download all covers (defaults to English for PAL games)");
            });

            ui.separator();
            ui.heading("🛠 Cheat Codes");

            ui.horizontal(|ui| {
                if ui.button("📥").clicked() {
                    txtcodes::spawn_download_cheats_task(&app_state.task_processor, &ui_buffers.config.contents.mount_point, app_state.games.clone().into_boxed_slice());
                }

                ui.label("Download cheats for all games (txt)");
            });

            ui.separator();
            ui.heading("🚿 Cleanup");

            ui.horizontal(|ui| {
                if ui.button("⏵").clicked() {
                    ui_buffers.action = Some(UiAction::RunNormalizePaths);
                }

                ui.label("Normalize paths (makes sure the game directories' layouts are correct)");
            });

            if cfg!(target_os = "macos") {
                ui.separator();
                ui.heading(" macOS");

                ui.horizontal(|ui| {
                    if ui.button("⏵").clicked() {
                        ui_buffers.run_dot_clean();
                    }

                    ui.label("Run dot_clean (remove hidden ._ files)");
                });
            }
        });
    });
}
