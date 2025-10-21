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
                    // Path
                    ui.label("📁 Path: ".to_string() + info.game_dir.to_str().unwrap_or("Unknown"));

                    ui.separator();

                    // Game ID
                    ui.label("🏷 ID: ".to_string() + info.game_id.as_ref());

                    // Embedded Title
                    ui.label("✏ Embedded Title: ".to_string() + &info.game_title);

                    // Is Wii
                    ui.label("🎾 Is Wii: ".to_string() + if info.is_wii { "Yes" } else { "No" });

                    // Is GameCube
                    ui.label(
                        "🎲 Is GameCube: ".to_string()
                            + if info.is_gamecube { "Yes" } else { "No" },
                    );

                    // Disc Number
                    ui.label(format!("🔢 Disc Number: {}", &info.disc_num));

                    // Disc Version
                    ui.label(format!("📌 Disc Version: {}", &info.disc_version));

                    // Region
                    ui.label("🌍 Region: ".to_string() + info.game_id.get_region_display());

                    ui.separator();

                    // Format
                    ui.label(format!("💿 Format: {}", &info.format));

                    // Compression
                    ui.label(format!("⬌ Compression: {}", &info.compression));

                    // Block Size
                    ui.label(format!("📏 Block Size: {}", &info.block_size));

                    // Decrypted
                    ui.label(
                        "🔒 Decrypted: ".to_string() + if info.decrypted { "Yes" } else { "No" },
                    );

                    // Needs Hash Recovery
                    ui.label(
                        "⚠ Needs Hash Recovery: ".to_string()
                            + if info.needs_hash_recovery {
                                "Yes"
                            } else {
                                "No"
                            },
                    );

                    // Lossless
                    ui.label("☑ Lossless: ".to_string() + if info.lossless { "Yes" } else { "No" });

                    // Disc Size
                    ui.label(format!("⚖ Disc Size: {}", &info.disc_size));

                    ui.separator();

                    // CRC32
                    ui.label("☑ CRC32: ".to_string() + &info.crc32);

                    // MD5
                    ui.label("☑ MD5: ".to_string() + &info.md5);

                    // SHA1
                    ui.label("☑ SHA1: ".to_string() + &info.sha1);

                    // XXH64
                    ui.label("☑ XXH64: ".to_string() + &info.xxh64);

                    ui.separator();

                    // Redump Verified
                    ui.label("🎯 Redump: ".to_string() + &info.redump_status);
                });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }

                if ui.button("📁 Open Directory").clicked()
                    && let Err(e) = open::that(&info.game_dir)
                {
                    app.toasts.error(e.to_string());
                }
            })
        });
    }

    if close {
        app.disc_info = None;
    }
}
