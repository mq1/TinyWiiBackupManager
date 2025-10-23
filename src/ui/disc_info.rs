// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games::GameID, verify};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("disc_info".into());
    let mut close = false;

    if let Some((display_title, info)) = &app.disc_info {
        modal.show(ctx, |ui| {
            ui.heading(format!("⏵ {}", display_title));

            // Path
            ui.label(format!("📁 Path: {}", info.game_dir.display()));

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.heading("⏵ Disc Header");

                    // Game ID
                    ui.label(format!("🏷 ID: {}", info.header.game_id_str()));

                    // Embedded Title
                    ui.label(format!(
                        "✏ Embedded Title: {}",
                        &info.header.game_title_str()
                    ));

                    // Region
                    ui.label(format!(
                        "🌐 Region: {}",
                        GameID(info.header.game_id).get_region_display()
                    ));

                    // Is Wii
                    ui.label(format!(
                        "🎾 Is Wii: {}",
                        if info.header.is_wii() { "Yes" } else { "No" }
                    ));

                    // Is GameCube
                    ui.label(format!(
                        "🎲 Is GameCube: {}",
                        if info.header.is_gamecube() {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Disc Number
                    ui.label(format!("🔢 Disc Number: {}", &info.header.disc_num));

                    // Disc Version
                    ui.label(format!("📌 Disc Version: {}", &info.header.disc_version));

                    ui.separator();

                    ui.heading("⏵ Disc Meta");

                    // Format
                    ui.label(format!("💿 Format: {}", &info.meta.format));

                    // Compression
                    ui.label(format!("⬌ Compression: {}", &info.meta.compression));

                    // Block Size
                    ui.label(format!(
                        "📏 Block Size: {}",
                        &info.meta.block_size.unwrap_or(0)
                    ));

                    // Decrypted
                    ui.label(format!(
                        "🔐 Decrypted: {}",
                        if info.meta.decrypted { "Yes" } else { "No" },
                    ));

                    // Needs Hash Recovery
                    ui.label(format!(
                        "⚠ Needs Hash Recovery: {}",
                        if info.meta.needs_hash_recovery {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Lossless
                    ui.label(format!(
                        "☑ Lossless: {}",
                        if info.meta.lossless { "Yes" } else { "No" }
                    ));

                    // Disc Size
                    ui.label(format!(
                        "⚖ Disc Size: {}",
                        &info.meta.disc_size.unwrap_or(0)
                    ));

                    ui.separator();

                    ui.heading("⏵ Disc Hashes");

                    // CRC32
                    ui.label(format!("☑ CRC32: {:02x}", &info.meta.crc32.unwrap_or(0)));

                    // MD5
                    ui.label(format!(
                        "☑ MD5: {}",
                        hex::encode(info.meta.md5.unwrap_or([0; 16]))
                    ));

                    // SHA1
                    ui.label(format!(
                        "☑ SHA1: {}",
                        hex::encode(&info.meta.sha1.unwrap_or([0; 20]))
                    ));

                    // XXH64
                    ui.label(format!("☑ XXH64: {:02x}", &info.meta.xxh64.unwrap_or(0)));
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

                // Integrity check button
                if ui
                    .button("✅ Verify Hashes")
                    .on_hover_text("Integrity Check")
                    .clicked()
                {
                    verify::spawn_verify_game_task(info.game_dir.clone(), &app.task_processor);
                }
            })
        });
    }

    if close {
        app.disc_info = None;
    }
}
