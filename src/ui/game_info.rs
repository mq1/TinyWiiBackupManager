// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{checksum, disc_info::DiscInfo, txtcodes, wiitdb::GameInfo};
use capitalize::Capitalize;
use eframe::egui;
use itertools::Itertools;

pub fn update(
    ctx: &egui::Context,
    app: &App,
    game_i: u16,
    disc_info: &Option<DiscInfo>,
    game_info: &Option<GameInfo>,
) {
    let game = &app.games[game_i as usize];

    egui::Modal::new("game_info".into()).show(ctx, |ui| {
        ui.heading(format!(
            "{} {}",
            egui_phosphor::regular::SWORD,
            game.display_title
        ));
        ui.label(format!(
            "{} Path: {}",
            egui_phosphor::regular::FOLDER,
            game.path.display()
        ));

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                ui.heading(format!(
                    "{} Disc Header",
                    egui_phosphor::regular::CARET_RIGHT
                ));

                if let Some(disc_info) = &disc_info {
                    // Game ID
                    ui.label(format!(
                        "{} ID: {}",
                        egui_phosphor::regular::TAG,
                        disc_info.id.as_str()
                    ));

                    // Embedded Title
                    ui.label(format!(
                        "{} Embedded Title: {}",
                        egui_phosphor::regular::NOTE_PENCIL,
                        &disc_info.title
                    ));

                    // Region
                    ui.label(format!(
                        "{} Region (inferred from ID): {}",
                        egui_phosphor::regular::GLOBE,
                        disc_info.id.get_region_display()
                    ));

                    // Is Wii
                    ui.label(format!(
                        "{} Is Wii: {}",
                        egui_phosphor::regular::HAND_DEPOSIT,
                        if disc_info.is_wii { "Yes" } else { "No" }
                    ));

                    // Is GameCube
                    ui.label(format!(
                        "{} Is GameCube: {}",
                        egui_phosphor::regular::GAME_CONTROLLER,
                        if disc_info.is_gc { "Yes" } else { "No" },
                    ));

                    // Disc Number
                    ui.label(format!(
                        "{} Disc Number: {}",
                        egui_phosphor::regular::HASH,
                        &disc_info.disc_num
                    ));

                    // Disc Version
                    ui.label(format!(
                        "{} Disc Version: {}",
                        egui_phosphor::regular::PUSH_PIN,
                        &disc_info.disc_version
                    ));

                    ui.separator();

                    ui.heading(format!("{} Disc Meta", egui_phosphor::regular::CARET_RIGHT));

                    // Format
                    ui.label(format!(
                        "{} Format: {}",
                        egui_phosphor::regular::DISC,
                        &disc_info.format
                    ));

                    // Compression
                    ui.label(format!(
                        "{} Compression: {}",
                        egui_phosphor::regular::FILE_ARCHIVE,
                        disc_info.compression
                    ));

                    // Block Size
                    ui.label(format!(
                        "{} Block Size: {}",
                        egui_phosphor::regular::RULER,
                        &disc_info.block_size
                    ));

                    // Decrypted
                    ui.label(format!(
                        "{} Decrypted: {}",
                        egui_phosphor::regular::LOCK_OPEN,
                        if disc_info.decrypted { "Yes" } else { "No" },
                    ));

                    // Needs Hash Recovery
                    ui.label(format!(
                        "{} Needs Hash Recovery: {}",
                        egui_phosphor::regular::WARNING,
                        if disc_info.needs_hash_recovery {
                            "Yes"
                        } else {
                            "No"
                        },
                    ));

                    // Lossless
                    ui.label(format!(
                        "{} Lossless: {}",
                        egui_phosphor::regular::SEAL_CHECK,
                        if disc_info.lossless { "Yes" } else { "No" }
                    ));

                    // Disc Size
                    ui.label(format!(
                        "{} Disc Size: {}",
                        egui_phosphor::regular::SCALES,
                        &disc_info.disc_size
                    ));

                    ui.separator();

                    ui.heading(format!(
                        "{} Disc Meta (Hashes)",
                        egui_phosphor::regular::CARET_RIGHT
                    ));

                    // CRC32
                    if let Some(crc32) = disc_info.crc32 {
                        ui.label(format!(
                            "{} CRC32: {:02x}",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE,
                            &crc32
                        ));
                    } else {
                        ui.label(format!(
                            "{} CRC32: N/A",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE
                        ));
                    }

                    // MD5
                    if let Some(md5) = disc_info.md5 {
                        ui.label(format!(
                            "{} MD5: {}",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE,
                            hex::encode(md5)
                        ));
                    } else {
                        ui.label(format!(
                            "{} MD5: N/A",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE
                        ));
                    }

                    // SHA1
                    if let Some(sha1) = disc_info.sha1 {
                        ui.label(format!(
                            "{} SHA1: {}",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE,
                            hex::encode(sha1)
                        ));
                    } else {
                        ui.label(format!(
                            "{} SHA1: N/A",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE
                        ));
                    }

                    // XXH64
                    if let Some(xxh64) = disc_info.xxh64 {
                        ui.label(format!(
                            "{} XXH64: {:02x}",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE,
                            &xxh64
                        ));
                    } else {
                        ui.label(format!(
                            "{} XXH64: N/A",
                            egui_phosphor::regular::FINGERPRINT_SIMPLE
                        ));
                    }
                } else {
                    ui.label(format!(
                        "{} Unable to read disc info",
                        egui_phosphor::regular::WARNING
                    ));
                }

                ui.separator();

                ui.heading(format!(
                    "{} Game Info from wiitdb.xml",
                    egui_phosphor::regular::CARET_RIGHT
                ));

                if let Some(game_info) = &game_info {
                    // Name
                    ui.label(format!(
                        "{} Name: {}",
                        egui_phosphor::regular::NOTE_PENCIL,
                        &game_info.name
                    ));

                    // Region
                    ui.label(format!(
                        "{} Region: {}",
                        egui_phosphor::regular::GLOBE,
                        &game_info.region.as_str()
                    ));

                    // Languages
                    ui.label(format!(
                        "{} Languages: {}",
                        egui_phosphor::regular::TRANSLATE,
                        &game_info.languages.iter().map(|l| l.as_str()).join(", ")
                    ));

                    // Developer
                    ui.label(format!(
                        "{} Developer: {}",
                        egui_phosphor::regular::USER,
                        game_info.developer.as_deref().unwrap_or("Unknown")
                    ));

                    // Publisher
                    ui.label(format!(
                        "{} Publisher: {}",
                        egui_phosphor::regular::BUILDING,
                        game_info.publisher.as_deref().unwrap_or("Unknown")
                    ));

                    // Date
                    ui.label(format!(
                        "{} Date: {}-{}-{}",
                        egui_phosphor::regular::CALENDAR,
                        &game_info.date.year,
                        &game_info.date.month,
                        &game_info.date.day
                    ));

                    // Genres
                    ui.label(format!(
                        "{} Genre(s): {}",
                        egui_phosphor::regular::SWORD,
                        &game_info.genre.join(", ")
                    ));

                    // Rating
                    ui.label(format!(
                        "{} Rating: {} {}",
                        egui_phosphor::regular::BABY,
                        &game_info.rating.r#type,
                        &game_info.rating.value
                    ));

                    // Wifi
                    ui.label(format!(
                        "{} WiFi: {} Players • {}",
                        egui_phosphor::regular::WIFI_HIGH,
                        &game_info.wifi.players,
                        game_info.wifi.features.join(", ")
                    ));

                    // Input
                    ui.label(format!(
                        "{} Input: {} Players • {}",
                        egui_phosphor::regular::JOYSTICK,
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
                    ui.label(format!(
                        "{} Unable to read game info",
                        egui_phosphor::regular::WARNING
                    ));
                }

                if let Some(disc_info) = &disc_info
                    && let Some(game_info) = &game_info
                    && !game_info.roms.is_empty()
                    && let Some(crc32) = disc_info.crc32
                {
                    ui.separator();

                    if game_info
                        .roms
                        .iter()
                        .filter_map(|r| r.crc)
                        .any(|db_crc| db_crc == crc32)
                    {
                        ui.label(format!(
                            "{} Redump: Verified",
                            egui_phosphor::regular::SEAL_CHECK
                        ));
                    } else {
                        ui.label(format!(
                            "{} Redump: Not Verified",
                            egui_phosphor::regular::SEAL_QUESTION
                        ));
                    }
                }
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui
                .button(format!("{} Close", egui_phosphor::regular::X))
                .clicked()
            {
                app.send_msg(Message::CloseModal);
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui
                .button(format!("{} Open Directory", egui_phosphor::regular::FOLDER))
                .clicked()
            {
                app.send_msg(Message::OpenGameDir(game_i));
            }

            // Strip update partition button
            if disc_info
                .as_ref()
                .is_some_and(|disc_info| disc_info.is_worth_stripping)
            {
                if ui
                    .button(format!(
                        "{} Remove Update Partition",
                        egui_phosphor::regular::FILE_DASHED
                    ))
                    .on_hover_text(format!(
                        "Removes the update partition from the disc\n{}This is irreversible!",
                        egui_phosphor::regular::WARNING
                    ))
                    .clicked()
                {
                    println!("Placeholder");
                }
            }

            // Integrity check button
            let has_embedded_crc32 = disc_info
                .as_ref()
                .is_some_and(|disc_info| disc_info.crc32.is_some());

            if (has_embedded_crc32 || game_info.is_some())
                && ui
                    .button(format!("{} Verify Hashes", egui_phosphor::regular::CHECKS))
                    .on_hover_text("Integrity Check")
                    .clicked()
            {
                checksum::spawn_checksum_task(app, game.path.clone(), game_info.clone());
            }

            // Download cheats
            if ui
                .button(format!(
                    "{} Download Cheats",
                    egui_phosphor::regular::CLOUD_ARROW_DOWN
                ))
                .clicked()
            {
                txtcodes::spawn_download_cheats_task(app, game);
            }
        });
    });
}
