use crate::components::fake_link::fake_link;
use crate::game::{ConsoleType, Game, VerificationStatus};
use crate::messages::BackgroundMessage;
use crate::util::redump;
use eframe::egui::{self, RichText};
use size::Size;

pub fn ui_game_info_window(
    ctx: &egui::Context,
    game: &mut Game,
    open: &mut bool,
    sender: egui_inbox::UiInboxSender<BackgroundMessage>,
) {
    egui::Window::new(game.display_title.clone())
        .open(open)
        .show(ctx, |ui| {
            ui_game_info_content(ui, game, sender);
        });
}

fn ui_game_info_content(
    ui: &mut egui::Ui,
    game: &mut Game,
    sender: egui_inbox::UiInboxSender<BackgroundMessage>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("📝 Title:").strong());
        ui.label(RichText::new(&game.title));
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🆔 Game ID:").strong());
        ui.label(RichText::new(&game.id).monospace());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🎮 Console:").strong());
        ui.label(match game.console {
            ConsoleType::GameCube => "GameCube",
            ConsoleType::Wii => "Wii",
        });
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🌍 Region:").strong());
        ui.label(&game.language);
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("💾 Size on disk:").strong());
        ui.label(Size::from_bytes(game.size).to_string());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("📁 Path:").strong());
        if fake_link(ui, &game.path.display().to_string()).clicked()
            && let Err(e) = open::that(&game.path)
        {
            let _ = sender.send(BackgroundMessage::Error(anyhow::anyhow!(e)));
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🌐 GameTDB:").strong());
        ui.hyperlink(&game.info_url);
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("💿 Disc Metadata");
    ui.add_space(5.0);
    if let Some(meta) = game.load_disc_meta() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("💿 Format:").strong());
            ui.label(meta.format.to_string());
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("📦 Compression:").strong());
            ui.label(meta.compression.to_string());
        });

        if let Some(block_size) = meta.block_size {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🧱 Block size:").strong());
                ui.label(Size::from_bytes(block_size).to_string());
            });
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new("✅ Lossless:").strong());
            if meta.lossless {
                ui.colored_label(egui::Color32::DARK_GREEN, "Yes");
            } else {
                ui.colored_label(egui::Color32::DARK_RED, "No");
            }
        });

        if let Some(disc_size) = meta.disc_size {
            ui.horizontal(|ui| {
                ui.label(RichText::new("💾 Original size:").strong());
                ui.label(Size::from_bytes(disc_size).to_string());
            });
        }
    } else {
        ui.label(RichText::new("Couldn't read disc metadata").color(ui.visuals().warn_fg_color));
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("🔍 Integrity");
    ui.add_space(5.0);

    // Show Redump status
    let verification_status = game.get_verification_status();
    match verification_status {
        VerificationStatus::EmbeddedMatch(redump) => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📀 Redump:").strong());
                ui.colored_label(
                    egui::Color32::from_rgb(255, 200, 0),
                    format!("{} ⚡", redump.name),
                );
            });
            ui.label(RichText::new("Embedded hashes match").italics());
        }
        VerificationStatus::FullyVerified(redump, _) => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📀 Redump:").strong());
                ui.colored_label(egui::Color32::DARK_GREEN, format!("{} ✅", redump.name));
            });
            ui.label(RichText::new("Fully verified").italics());
        }
        VerificationStatus::Failed(msg, _) => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📀 Redump:").strong());
                ui.colored_label(egui::Color32::DARK_RED, format!("{} ❌", msg));
            });
        }
        VerificationStatus::NotVerified => {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📀 Redump:").strong());
                ui.label("Not checked");
            });
        }
    }

    ui.add_space(5.0);

    // Check verification status and show calculated hashes if available
    let (hashes, redump) = match verification_status {
        VerificationStatus::FullyVerified(r, h) => (Some(h), Some(r.clone())),
        VerificationStatus::Failed(_, Some(h)) => {
            // Try to find redump entry for partial matches
            let redump = h.crc32.and_then(redump::find_by_crc32);
            (Some(h), redump)
        }
        _ => (None, None),
    };

    if let Some(hashes) = hashes {
        // Show calculated hashes with color coding
        ui.label(RichText::new("Calculated hashes:").strong());

        if let Some(crc32) = hashes.crc32 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("  CRC32:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                let hash_text = format!("{:08x}", crc32);
                let hash_color = if let Some(ref r) = redump {
                    if r.crc32 == crc32 {
                        egui::Color32::DARK_GREEN
                    } else {
                        egui::Color32::DARK_RED
                    }
                } else {
                    ui.visuals().text_color()
                };
                ui.colored_label(
                    hash_color,
                    RichText::new(hash_text).text_style(egui::TextStyle::Monospace),
                );
            });
        }

        if let Some(sha1) = hashes.sha1 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("  SHA-1:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                let hash_text = hex::encode(sha1);
                let hash_color = if let Some(ref r) = redump {
                    if r.sha1 == sha1 {
                        egui::Color32::DARK_GREEN
                    } else {
                        egui::Color32::DARK_RED
                    }
                } else {
                    ui.visuals().text_color()
                };
                ui.colored_label(
                    hash_color,
                    RichText::new(hash_text).text_style(egui::TextStyle::Monospace),
                );
            });
        }

        if let Some(xxhash64) = hashes.xxh64 {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("  XXH64:")
                        .text_style(egui::TextStyle::Monospace)
                        .strong(),
                );
                // XXH64 is not in Redump, so no color coding
                ui.label(
                    RichText::new(format!("{:016x}", xxhash64))
                        .text_style(egui::TextStyle::Monospace),
                );
            });
        }
    } else if let Some(meta) = game.load_disc_meta() {
        // Show embedded hashes if available
        let has_integrity_info = meta.crc32.is_some()
            || meta.md5.is_some()
            || meta.sha1.is_some()
            || meta.xxh64.is_some();

        if has_integrity_info {
            ui.label(RichText::new("Embedded hashes:").strong());

            if let Some(crc32) = meta.crc32 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("  CRC32:")
                            .text_style(egui::TextStyle::Monospace)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("{:08x}", crc32))
                            .text_style(egui::TextStyle::Monospace),
                    );
                });
            }

            if let Some(md5) = meta.md5 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("  MD5:")
                            .text_style(egui::TextStyle::Monospace)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(hex::encode(md5)).text_style(egui::TextStyle::Monospace),
                    );
                });
            }

            if let Some(sha1) = meta.sha1 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("  SHA-1:")
                            .text_style(egui::TextStyle::Monospace)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(hex::encode(sha1)).text_style(egui::TextStyle::Monospace),
                    );
                });
            }

            if let Some(xxhash64) = meta.xxh64 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("  XXH64:")
                            .text_style(egui::TextStyle::Monospace)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!("{:016x}", xxhash64))
                            .text_style(egui::TextStyle::Monospace),
                    );
                });
            }
        } else {
            ui.label("No embedded integrity info available");
        }
    } else {
        ui.label("No integrity info available");
    }

    ui.add_space(10.0);

    // Add Verify button
    if ui.button("🔍 Verify Disc").clicked() {
        // Send a message to start verification of this single game
        let _ = sender.send(BackgroundMessage::StartSingleVerification(Box::new(
            game.clone(),
        )));
    }
}
