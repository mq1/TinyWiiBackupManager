// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::{ConsoleType, Game};
use crate::gui::fake_link::fake_link;
use crate::gui::game_checks::ui_game_checks;
use crate::messages::BackgroundMessage;
use anyhow::anyhow;
use eframe::egui::{self, Id, Image, RichText};
use egui_inbox::UiInboxSender;
use size::Size;

pub fn ui_game_info_window(
    ctx: &egui::Context,
    game: &mut Game,
    sender: &mut UiInboxSender<BackgroundMessage>,
) {
    if game.info_opened && game.disc_meta.is_none() {
        game.refresh_meta();
    }

    let game_clone = game.clone();

    egui::Window::new(&game.title)
        .id(Id::new(game.id))
        .open(&mut game.info_opened)
        .show(ctx, |ui| {
            ui_game_info_content(ui, game_clone, sender);
        });
}

fn ui_game_info_content(
    ui: &mut egui::Ui,
    game: Game,
    sender: &mut UiInboxSender<BackgroundMessage>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Title:", egui_phosphor::regular::NOTE)).strong());
        ui.label(RichText::new(&game.title));
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Game ID:",
                egui_phosphor::regular::IDENTIFICATION_CARD
            ))
            .strong(),
        );
        ui.label(RichText::new(game.id_str()).monospace());
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Console:",
                egui_phosphor::regular::GAME_CONTROLLER
            ))
            .strong(),
        );
        ui.label(match game.console {
            ConsoleType::GameCube => "GameCube",
            ConsoleType::Wii => "Wii",
        });
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Size on disk:",
                egui_phosphor::regular::HARD_DRIVE
            ))
            .strong(),
        );
        ui.label(Size::from_bytes(game.size).to_string());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Path:", egui_phosphor::regular::FOLDER)).strong());
        if fake_link(ui, &game.path.display().to_string()).clicked()
            && let Err(e) = open::that(&game.path)
        {
            let _ = sender.send(anyhow!(e).into());
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} GameTDB:", egui_phosphor::regular::LINK)).strong());
        ui.hyperlink(&game.info_url);
    });

    if let Some(info) = &game.info {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} Region:", egui_phosphor::regular::GLOBE)).strong());
            ui.label(info.region.to_string());
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} Languages:", egui_phosphor::regular::FLAG)).strong(),
            );
            for lang in &info.languages {
                ui.add(Image::new(lang.flag()).max_width(20.0));
            }
        });
    }

    if let Some(meta) = &game.disc_meta
        && let Ok(meta) = &meta
    {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.heading(format!(
            "{} Disc Metadata",
            egui_phosphor::regular::FLOPPY_DISK
        ));
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} Format:", egui_phosphor::regular::DISC)).strong());
            ui.label(meta.format.to_string());
        });

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} Compression:", egui_phosphor::regular::PACKAGE)).strong(),
            );
            ui.label(meta.compression.to_string());
        });

        if let Some(block_size) = meta.block_size {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} Block size:", egui_phosphor::regular::RULER))
                        .strong(),
                );
                ui.label(Size::from_bytes(block_size).to_string());
            });
        }

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} Lossless:", egui_phosphor::regular::CHECK)).strong(),
            );
            if meta.lossless {
                ui.colored_label(egui::Color32::DARK_GREEN, "Yes");
            } else {
                ui.colored_label(egui::Color32::DARK_RED, "No");
            }
        });

        if let Some(disc_size) = meta.disc_size {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} Original size:",
                        egui_phosphor::regular::FLOPPY_DISK
                    ))
                    .strong(),
                );
                ui.label(Size::from_bytes(disc_size).to_string());
            });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        ui.heading(format!(
            "{} Integrity Metadata",
            egui_phosphor::regular::MAGNIFYING_GLASS
        ));
        ui.add_space(5.0);

        if let Some(crc32) = meta.crc32 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("CRC32:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{:08x}", crc32)).text_style(egui::TextStyle::Monospace),
                );

                ui_game_checks(ui, &game);
            });
        }

        if let Some(md5) = meta.md5 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("MD5:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                ui.label(RichText::new(hex::encode(md5)).text_style(egui::TextStyle::Monospace));
            });
        }

        if let Some(sha1) = meta.sha1 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("SHA-1:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                ui.label(RichText::new(hex::encode(sha1)).text_style(egui::TextStyle::Monospace));
            });
        }

        if let Some(xxhash64) = meta.xxh64 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("XXH64:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                ui.label(
                    RichText::new(format!("{:016x}", xxhash64))
                        .text_style(egui::TextStyle::Monospace),
                );
            });
        }
    }
}
