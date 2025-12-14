// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{checksum, txtcodes};
use capitalize::Capitalize;
use eframe::egui;
use egui_phosphor::regular as ph;
use itertools::Itertools;

pub fn update(ctx: &egui::Context, app: &App, game_i: u16) {
    let game = &app.games[game_i as usize];
    let disc_info = &app.current_disc_info;

    let game_info = if let Some(wiitdb) = &app.wiitdb
        && let Some(i) = app.current_game_info
    {
        Some(&wiitdb.games[i as usize])
    } else {
        None
    };

    egui::Modal::new("game_info".into()).show(ctx, |ui| {
        ui.set_height(ctx.available_rect().height() - 80.);
        ui.set_width(700.);

        ui.horizontal(|ui| {
            ui.heading(format!("{} {}", ph::SWORD, game.display_title));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                let url = format!("https://www.gametdb.com/Wii/{}", game.id.as_str());

                ui.hyperlink_to(format!("{} GameTDB", ph::INFO), &url)
                    .on_hover_text(url)
            });
        });

        ui.label(format!("{} Path: {}", ph::FOLDER, game.path.display()));

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.heading(format!("{} Disc Header", ph::CARET_RIGHT));

            if let Some(disc_info) = &disc_info {
                // Game ID
                ui.label(format!("{} ID: {}", ph::TAG, disc_info.id.as_str()));

                // Embedded Title
                ui.label(format!(
                    "{} Embedded Title: {}",
                    ph::NOTE_PENCIL,
                    &disc_info.title
                ));

                // Region
                ui.label(format!(
                    "{} Region (inferred from ID): {}",
                    ph::GLOBE,
                    disc_info.id.get_region_display()
                ));

                // Is Wii
                ui.label(format!(
                    "{} Is Wii: {}",
                    ph::HAND_DEPOSIT,
                    if disc_info.is_wii { "Yes" } else { "No" }
                ));

                // Is GameCube
                ui.label(format!(
                    "{} Is GameCube: {}",
                    ph::GAME_CONTROLLER,
                    if disc_info.is_gc { "Yes" } else { "No" },
                ));

                // Disc Number
                ui.label(format!("{} Disc Number: {}", ph::HASH, &disc_info.disc_num));

                // Disc Version
                ui.label(format!(
                    "{} Disc Version: {}",
                    ph::PUSH_PIN,
                    &disc_info.disc_version
                ));

                ui.separator();

                ui.heading(format!("{} Disc Meta", ph::CARET_RIGHT));

                // Format
                ui.label(format!("{} Format: {}", ph::DISC, &disc_info.format));

                // Compression
                ui.label(format!(
                    "{} Compression: {}",
                    ph::FILE_ARCHIVE,
                    disc_info.compression
                ));

                // Block Size
                ui.label(format!(
                    "{} Block Size: {}",
                    ph::RULER,
                    &disc_info.block_size
                ));

                // Decrypted
                ui.label(format!(
                    "{} Decrypted: {}",
                    ph::LOCK_OPEN,
                    if disc_info.decrypted { "Yes" } else { "No" },
                ));

                // Needs Hash Recovery
                ui.label(format!(
                    "{} Needs Hash Recovery: {}",
                    ph::WARNING,
                    if disc_info.needs_hash_recovery {
                        "Yes"
                    } else {
                        "No"
                    },
                ));

                // Lossless
                ui.label(format!(
                    "{} Lossless: {}",
                    ph::SEAL_CHECK,
                    if disc_info.lossless { "Yes" } else { "No" }
                ));

                // Disc Size
                ui.label(format!(
                    "{} Disc Size: {}",
                    ph::SCALES,
                    &disc_info.disc_size
                ));

                ui.separator();

                ui.heading(format!("{} Disc Meta (Hashes)", ph::CARET_RIGHT));

                // CRC32
                if let Some(crc32) = disc_info.crc32 {
                    ui.label(format!("{} CRC32: {:02x}", ph::FINGERPRINT_SIMPLE, &crc32));
                } else {
                    ui.label(format!("{} CRC32: N/A", ph::FINGERPRINT_SIMPLE));
                }

                // MD5
                if let Some(md5) = disc_info.md5 {
                    ui.label(format!(
                        "{} MD5: {}",
                        ph::FINGERPRINT_SIMPLE,
                        hex::encode(md5)
                    ));
                } else {
                    ui.label(format!("{} MD5: N/A", ph::FINGERPRINT_SIMPLE));
                }

                // SHA1
                if let Some(sha1) = disc_info.sha1 {
                    ui.label(format!(
                        "{} SHA1: {}",
                        ph::FINGERPRINT_SIMPLE,
                        hex::encode(sha1)
                    ));
                } else {
                    ui.label(format!("{} SHA1: N/A", ph::FINGERPRINT_SIMPLE));
                }

                // XXH64
                if let Some(xxh64) = disc_info.xxh64 {
                    ui.label(format!("{} XXH64: {:02x}", ph::FINGERPRINT_SIMPLE, &xxh64));
                } else {
                    ui.label(format!("{} XXH64: N/A", ph::FINGERPRINT_SIMPLE));
                }
            } else {
                ui.label(format!("{} Unable to read disc info", ph::WARNING));
            }

            ui.separator();

            ui.heading(format!("{} Game Info from wiitdb.xml", ph::CARET_RIGHT));

            if let Some(game_info) = &game_info {
                // Name
                ui.label(format!("{} Name: {}", ph::NOTE_PENCIL, &game_info.name));

                // Region
                ui.label(format!(
                    "{} Region: {}",
                    ph::GLOBE,
                    &game_info.region.as_str()
                ));

                // Languages
                ui.label(format!(
                    "{} Languages: {}",
                    ph::TRANSLATE,
                    &game_info.languages.iter().map(|l| l.as_str()).join(", ")
                ));

                // Developer
                ui.label(format!(
                    "{} Developer: {}",
                    ph::USER,
                    game_info.developer.as_deref().unwrap_or("Unknown")
                ));

                // Publisher
                ui.label(format!(
                    "{} Publisher: {}",
                    ph::BUILDING,
                    game_info.publisher.as_deref().unwrap_or("Unknown")
                ));

                // Date
                ui.label(format!(
                    "{} Date: {}-{}-{}",
                    ph::CALENDAR,
                    &game_info.date.year,
                    &game_info.date.month,
                    &game_info.date.day
                ));

                // Genres
                ui.label(format!(
                    "{} Genre(s): {}",
                    ph::SWORD,
                    &game_info.genre.join(", ")
                ));

                // Rating
                ui.label(format!(
                    "{} Rating: {} {}",
                    ph::BABY,
                    &game_info.rating.r#type,
                    &game_info.rating.value
                ));

                // Wifi
                ui.label(format!(
                    "{} WiFi: {} Players • {}",
                    ph::WIFI_HIGH,
                    &game_info.wifi.players,
                    game_info.wifi.features.join(", ")
                ));

                // Input
                ui.label(format!(
                    "{} Input: {} Players • {}",
                    ph::JOYSTICK,
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
                ui.label(format!("{} Unable to read game info", ph::WARNING));
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
                    ui.label(format!("{} Redump: Verified", ph::SEAL_CHECK));
                } else {
                    ui.label(format!("{} Redump: Not Verified", ph::SEAL_QUESTION));
                }
            }
        });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button(format!("{} Close", ph::X)).clicked() {
                app.send_msg(Message::CloseModal);
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui
                .button(format!("{} Open Directory", ph::FOLDER))
                .clicked()
            {
                app.send_msg(Message::OpenGameDir(game_i));
            }

            // Strip update partition button
            if disc_info
                .as_ref()
                .is_some_and(|disc_info| disc_info.is_worth_stripping)
                && ui
                    .button(format!("{} Remove Update Partition", ph::FILE_DASHED))
                    .on_hover_text(format!(
                        "Removes the update partition from the disc\n{}This is irreversible!",
                        ph::WARNING
                    ))
                    .clicked()
            {
                app.send_msg(Message::StripGame);
            }

            // Integrity check button
            if let Some(disc_info) = disc_info
                && (disc_info.crc32.is_some() || game_info.is_some())
                && ui
                    .button(format!("{} Verify Hashes", ph::SEAL_CHECK))
                    .on_hover_text("Integrity Check")
                    .clicked()
            {
                checksum::spawn_checksum_task(app, disc_info.clone(), game_info);
            }

            // Download cheats
            if ui
                .button(format!("{} Download Cheats", ph::CLOUD_ARROW_DOWN))
                .clicked()
            {
                txtcodes::spawn_download_cheats_task(app, game);
            }
        });
    });
}
