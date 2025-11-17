// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{checksum, disc_info::DiscInfo, games::GameID, wiitdb::GameInfo};
use capitalize::Capitalize;
use eframe::egui;
use itertools::Itertools;

pub fn update(
    ctx: &egui::Context,
    app: &mut App,
    game_i: u16,
    disc_info: &Option<DiscInfo>,
    game_info: &Option<GameInfo>,
) {
    egui::Modal::new("game_info".into()).show(ctx, |ui| {
        {
            let game = &app.games[game_i as usize];
            ui.heading(format!("⏵ {}", game.display_title));
            ui.label(format!("📁 Path: {}", game.path.display()));
        }

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                ui.heading("⏵ Disc Info");

                if let Some(disc_info) = &disc_info {
                    // Game ID
                    ui.label(format!("🏷 ID: {}", disc_info.header.game_id_str()));

                    // Embedded Title
                    ui.label(format!(
                        "✏ Embedded Title: {}",
                        &disc_info.header.game_title_str()
                    ));

                    // Region
                    ui.label(format!(
                        "🌐 Region (inferred from ID): {}",
                        disc_info.header.game_id.get_region_display()
                    ));

                    // Is Wii
                    ui.label(format!(
                        "🎾 Is Wii: {}",
                        if disc_info.header.is_wii() {
                            "Yes"
                        } else {
                            "No"
                        }
                    ));

                    // Is GameCube
                    ui.label(format!(
                        "🎲 Is GameCube: {}",
                        if disc_info.header.is_gamecube() {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Disc Number
                    ui.label(format!("🔢 Disc Number: {}", &disc_info.header.disc_num));

                    // Disc Version
                    ui.label(format!(
                        "📌 Disc Version: {}",
                        &disc_info.header.disc_version
                    ));

                    ui.separator();

                    // Format
                    ui.label(format!("💿 Format: {}", &disc_info.meta.format));

                    // Compression
                    ui.label(format!("⬌ Compression: {}", &disc_info.meta.compression));

                    // Block Size
                    ui.label(format!(
                        "📏 Block Size: {}",
                        &disc_info.meta.block_size.unwrap_or(0)
                    ));

                    // Decrypted
                    ui.label(format!(
                        "🔐 Decrypted: {}",
                        if disc_info.meta.decrypted {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Needs Hash Recovery
                    ui.label(format!(
                        "⚠ Needs Hash Recovery: {}",
                        if disc_info.meta.needs_hash_recovery {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Lossless
                    ui.label(format!(
                        "☑ Lossless: {}",
                        if disc_info.meta.lossless { "Yes" } else { "No" }
                    ));

                    // Disc Size
                    ui.label(format!(
                        "⚖ Disc Size: {}",
                        &disc_info.meta.disc_size.unwrap_or(0)
                    ));

                    ui.separator();

                    // CRC32
                    if let Some(crc32) = disc_info.meta.crc32 {
                        ui.label(format!("☑ CRC32: {:02x}", &crc32));
                    } else {
                        ui.label("☑ CRC32: N/A");
                    }

                    // MD5
                    if let Some(md5) = disc_info.meta.md5 {
                        ui.label(format!("☑ MD5: {}", hex::encode(md5)));
                    } else {
                        ui.label("☑ MD5: N/A");
                    }

                    // SHA1
                    if let Some(sha1) = disc_info.meta.sha1 {
                        ui.label(format!("☑ SHA1: {}", hex::encode(sha1)));
                    } else {
                        ui.label("☑ SHA1: N/A");
                    }

                    // XXH64
                    if let Some(xxh64) = disc_info.meta.xxh64 {
                        ui.label(format!("☑ XXH64: {:02x}", &xxh64));
                    } else {
                        ui.label("☑ XXH64: N/A");
                    }
                } else {
                    ui.label("⚠ Unable to read disc info");
                }

                ui.separator();

                ui.heading("⏵ Game Info from wiitdb.xml");

                if let Some(game_info) = &game_info {
                    // Name
                    ui.label(format!("✏ Name: {}", &game_info.name));

                    // Region
                    ui.label(format!("🌐 Region: {}", &game_info.region.as_str()));

                    // Languages
                    ui.label(format!(
                        "🌐 Languages: {}",
                        &game_info.languages.iter().map(|l| l.as_str()).join(", ")
                    ));

                    // Developer
                    ui.label(format!(
                        "👸 Developer: {}",
                        game_info.developer.as_deref().unwrap_or("Unknown")
                    ));

                    // Publisher
                    ui.label(format!(
                        "👸 Publisher: {}",
                        game_info.publisher.as_deref().unwrap_or("Unknown")
                    ));

                    // Date
                    ui.label(format!(
                        "📅 Date: {}-{}-{}",
                        &game_info.date.year, &game_info.date.month, &game_info.date.day
                    ));

                    // Genres
                    ui.label(format!("🎮 Genre(s): {}", &game_info.genre.join(", ")));

                    // Rating
                    ui.label(format!(
                        "☺ Rating: {} • {}",
                        &game_info.rating.r#type, &game_info.rating.value
                    ));

                    // Wifi
                    ui.label(format!(
                        "📶 WiFi: {} Players • {}",
                        &game_info.wifi.players,
                        game_info.wifi.features.join(", ")
                    ));

                    // Input
                    ui.label(format!(
                        "🕹 Input: {} Players • {}",
                        &game_info.input.players,
                        game_info
                            .input
                            .controls
                            .iter()
                            .map(|c| format!(
                                "{} ({})",
                                c.r#type.capitalize_first_only(),
                                if c.required { "Required" } else { "Optional" }
                            ))
                            .join(", ")
                    ));
                } else {
                    ui.label("⚠ Unable to read game info");
                }

                if let Some(disc_info) = &disc_info
                    && let Some(game_info) = &game_info
                    && !game_info.roms.is_empty()
                    && let Some(crc32) = disc_info.meta.crc32
                {
                    ui.separator();

                    if game_info
                        .roms
                        .iter()
                        .filter_map(|r| r.crc)
                        .any(|db_crc| db_crc == crc32)
                    {
                        ui.label("🎯 Redump: Verified");
                    } else {
                        ui.label("🎯 Redump: Not Verified");
                    }
                }
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button("❌ Close").clicked() {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui.button("📁 Open Directory").clicked() {
                app.open_game_dir(game_i);
            }

            // Integrity check button
            let has_embedded_crc32 = disc_info
                .as_ref()
                .map(|d| d.meta.crc32.is_some())
                .unwrap_or(false);

            if (has_embedded_crc32 || game_info.is_some())
                && ui
                    .button("✅ Verify Hashes")
                    .on_hover_text("Integrity Check")
                    .clicked()
            {
                checksum::spawn_checksum_task(
                    &app.task_processor,
                    app.games[game_i as usize].path.clone(),
                    game_info.clone(),
                );
            }
        });
    });
}
