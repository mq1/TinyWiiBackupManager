// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("disc_info".into());
    let mut close = false;

    if let Some((display_title, info)) = &app.disc_info {
        modal.show(ctx, |ui| {
            ui.heading(display_title);

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // Path
                        ui.horizontal(|ui| {
                            ui.label("📁");
                            ui.label("Path:");
                            ui.label(info.game_dir.to_str().unwrap_or_default());
                        });

                        ui.separator();

                        // Game ID
                        ui.horizontal(|ui| {
                            ui.label("🏷");
                            ui.label("ID:");
                            ui.label(info.game_id.as_ref());
                        });

                        // Embedded Title
                        ui.horizontal(|ui| {
                            ui.label("✏");
                            ui.label("Embedded Title:");
                            ui.label(&info.game_title);
                        });

                        // Is Wii
                        ui.horizontal(|ui| {
                            ui.label("🎾");
                            ui.label("Is Wii:");
                            ui.label(if info.is_wii { "Yes" } else { "No" });
                        });

                        // Is GameCube
                        ui.horizontal(|ui| {
                            ui.label("🎲");
                            ui.label("Is GameCube:");
                            ui.label(if info.is_gamecube { "Yes" } else { "No" });
                        });

                        // Disc Number
                        ui.horizontal(|ui| {
                            ui.label("🔢");
                            ui.label("Disc Number:");
                            ui.label(info.disc_num.to_string());
                        });

                        // Disc Version
                        ui.horizontal(|ui| {
                            ui.label("🏷");
                            ui.label("Disc Version:");
                            ui.label(info.disc_version.to_string());
                        });

                        // Region
                        ui.horizontal(|ui| {
                            ui.label("🌍");
                            ui.label("Region:");
                            ui.label(info.game_id.get_region_display());
                        });

                        ui.separator();

                        // Format
                        ui.horizontal(|ui| {
                            ui.label("💿");
                            ui.label("Format:");
                            ui.label(info.format.to_string());
                        });

                        // Compression
                        ui.horizontal(|ui| {
                            ui.label("⬌");
                            ui.label("Compression:");
                            ui.label(info.compression.to_string());
                        });

                        // Block Size
                        ui.horizontal(|ui| {
                            ui.label("📏");
                            ui.label("Block Size:");
                            ui.label(info.block_size.to_string());
                        });

                        // Decrypted
                        ui.horizontal(|ui| {
                            ui.label("🔒");
                            ui.label("Decrypted:");
                            ui.label(if info.decrypted { "Yes" } else { "No" });
                        });

                        // Needs Hash Recovery
                        ui.horizontal(|ui| {
                            ui.label("⚠");
                            ui.label("Needs Hash Recovery:");
                            ui.label(if info.needs_hash_recovery {
                                "Yes"
                            } else {
                                "No"
                            });
                        });

                        // Lossless
                        ui.horizontal(|ui| {
                            ui.label("☑");
                            ui.label("Lossless:");
                            ui.label(if info.lossless { "Yes" } else { "No" });
                        });

                        // Disc Size
                        ui.horizontal(|ui| {
                            ui.label("⚖");
                            ui.label("Disc Size:");
                            ui.label(info.disc_size.to_string());
                        });

                        ui.separator();

                        // CRC32
                        ui.horizontal(|ui| {
                            ui.label("☑");
                            ui.label("CRC32:");
                            ui.label(&info.crc32);
                        });

                        // MD5
                        ui.horizontal(|ui| {
                            ui.label("☑");
                            ui.label("MD5:");
                            ui.label(&info.md5);
                        });

                        // SHA1
                        ui.horizontal(|ui| {
                            ui.label("☑");
                            ui.label("SHA1:");
                            ui.label(&info.sha1);
                        });

                        // XXH64
                        ui.horizontal(|ui| {
                            ui.label("☑");
                            ui.label("XXH64:");
                            ui.label(&info.xxh64);
                        });
                    });
                });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }

                if ui.button("📁 Open Directory").clicked() {
                    if let Err(e) = open::that(&info.game_dir) {
                        app.toasts.error(e.to_string());
                    }
                }
            })
        });
    }

    if close {
        app.disc_info = None;
    }
}
