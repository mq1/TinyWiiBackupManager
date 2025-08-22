use crate::components::fake_link::fake_link;
use crate::error_handling::show_anyhow_error;
use crate::game::{ConsoleType, Game};
use anyhow::anyhow;
use eframe::egui::{self, RichText};
use size::Size;

pub fn ui_game_info_window(ctx: &egui::Context, game: &mut Game, open: &mut bool) {
    egui::Window::new(game.display_title.clone())
        .open(open)
        .show(ctx, |ui| {
            ui_game_info_content(ui, game);
        });
}

fn ui_game_info_content(ui: &mut egui::Ui, game: &mut Game) {
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
        if fake_link(ui, &game.path.display().to_string()).clicked() {
            if let Err(e) = open::that(&game.path) {
                show_anyhow_error("Error opening file manager", &anyhow!(e));
            }
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
    if let Some(ref meta) = game.load_disc_meta() {
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

    if let Some(ref meta) = game.load_disc_meta() {
        let has_integrity_info = meta.crc32.is_some()
            || meta.md5.is_some()
            || meta.sha1.is_some()
            || meta.xxh64.is_some();

        if has_integrity_info {
            if let Some(crc32) = meta.crc32 {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("CRC32:")
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
                        RichText::new("MD5:")
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
                        RichText::new("SHA-1:")
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
        } else {
            ui.label("No integrity info available");
        }
    } else {
        ui.label(RichText::new("Couldn't read disc metadata").color(ui.visuals().warn_fg_color));
    }
}
