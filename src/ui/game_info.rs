// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, games::GameID, verify, wiitdb::Language};
use eframe::egui::{self, ImageSource, include_image};

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("disc_info".into());
    let mut close = false;

    if let Some((game, disc_info, game_info)) = &app.game_info {
        modal.show(ctx, |ui| {
            ui.heading(format!("⏵ {}", game.display_title));

            // Path
            ui.label(format!("📁 Path: {}", game.path.display()));

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.heading("⏵ Disc Info");

                    match disc_info {
                        Ok(disc_info) => {
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
                                GameID(disc_info.header.game_id).get_region_display()
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
                            ui.label(format!(
                                "☑ CRC32: {:02x}",
                                &disc_info.meta.crc32.unwrap_or(0)
                            ));

                            // MD5
                            ui.label(format!(
                                "☑ MD5: {}",
                                hex::encode(disc_info.meta.md5.unwrap_or([0; 16]))
                            ));

                            // SHA1
                            ui.label(format!(
                                "☑ SHA1: {}",
                                hex::encode(&disc_info.meta.sha1.unwrap_or([0; 20]))
                            ));

                            // XXH64
                            ui.label(format!(
                                "☑ XXH64: {:02x}",
                                &disc_info.meta.xxh64.unwrap_or(0)
                            ));
                        }
                        Err(e) => {
                            ui.label(format!("⚠ Error: {}", e));
                        }
                    }

                    ui.separator();

                    ui.heading("⏵ Game Info");

                    match game_info {
                        Ok(game_info) => {
                            // Languages
                            ui.horizontal(|ui| {
                                ui.label("🌐 Languages: ");

                                for lang in &game_info.languages {
                                    ui.add(egui::Image::new(lang.get_icon()).max_height(14.0));
                                }
                            });
                        }
                        Err(e) => {
                            ui.label(format!("⚠ Error: {}", e));
                        }
                    }
                });

            ui.add_space(10.);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("❌ Close").clicked() {
                    close = true;
                }

                if ui.button("📁 Open Directory").clicked()
                    && let Err(e) = open::that(&game.path)
                {
                    app.toasts.error(e.to_string());
                }

                // Integrity check button
                if ui
                    .button("✅ Verify Hashes")
                    .on_hover_text("Integrity Check")
                    .clicked()
                {
                    verify::spawn_verify_game_task(game.path.clone(), &app.task_processor);
                }
            })
        });
    }

    if close {
        app.game_info = None;
    }
}

impl Language {
    pub fn get_icon(&self) -> ImageSource<'_> {
        match self {
            Language::En => include_image!("../../assets/flag-icons/gb-eng.svg"),
            Language::Fr => include_image!("../../assets/flag-icons/fr.svg"),
            Language::De => include_image!("../../assets/flag-icons/de.svg"),
            Language::Es => include_image!("../../assets/flag-icons/es.svg"),
            Language::It => include_image!("../../assets/flag-icons/it.svg"),
            Language::Ja => include_image!("../../assets/flag-icons/jp.svg"),
            Language::Nl => include_image!("../../assets/flag-icons/nl.svg"),
            Language::Se => include_image!("../../assets/flag-icons/se.svg"),
            Language::Dk => include_image!("../../assets/flag-icons/dk.svg"),
            Language::No => include_image!("../../assets/flag-icons/no.svg"),
            Language::Ko => include_image!("../../assets/flag-icons/kr.svg"),
            Language::Pt => include_image!("../../assets/flag-icons/pt.svg"),
            Language::Zhtw => include_image!("../../assets/flag-icons/tw.svg"),
            Language::Zhcn => include_image!("../../assets/flag-icons/cn.svg"),
            Language::Fi => include_image!("../../assets/flag-icons/fi.svg"),
            Language::Tr => include_image!("../../assets/flag-icons/tr.svg"),
            Language::Gr => include_image!("../../assets/flag-icons/gr.svg"),
            Language::Ru => include_image!("../../assets/flag-icons/ru.svg"),
            Language::Unknown => include_image!("../../assets/flag-icons/xx.svg"),
        }
    }
}
