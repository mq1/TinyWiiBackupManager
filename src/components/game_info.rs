use crate::game::Game;
use eframe::egui::{self, RichText};
use size::Size;

pub fn ui_game_info_window(ctx: &egui::Context, game: &Game, open: &mut bool) {
    egui::Window::new(format!("Game Info - {}", game.id))
        .open(open)
        .fixed_size([400., 400.])
        .show(ctx, |ui| {
            ui_game_info_content(ui, game);
        });
}

fn ui_game_info_content(ui: &mut egui::Ui, game: &Game) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Title:").strong());
        ui.label(RichText::new(&game.display_title));
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Game ID:").strong());
        ui.label(RichText::new(&game.id).monospace());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Console:").strong());
        ui.label(if game.is_gc { "GameCube" } else { "Wii" });
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Region:").strong());
        ui.label(&game.language);
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Size on disk:").strong());
        ui.label(Size::from_bytes(game.size).to_string());
    });

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Disc Metadata");
    ui.add_space(5.0);
    if let Some(ref meta) = game.disc_meta {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Format:").strong());
            ui.label(meta.format.to_string());
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Compression:").strong());
            ui.label(meta.compression.to_string());
        });

        if let Some(block_size) = meta.block_size {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Block size:").strong());
                ui.label(Size::from_bytes(block_size).to_string());
            });
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new("Lossless:").strong());
            if meta.lossless {
                ui.colored_label(egui::Color32::DARK_GREEN, "Yes");
            } else {
                ui.colored_label(egui::Color32::DARK_RED, "No");
            }
        });

        if let Some(disc_size) = meta.disc_size {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Original size:").strong());
                ui.label(Size::from_bytes(disc_size).to_string());
            });
        }

        ui.add_space(10.0);

        ui.heading("Integrity");
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
    } else {
        ui.label(RichText::new("Couldn't read disc metadata").color(ui.visuals().warn_fg_color));
    }

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.hyperlink_to("🌐 View on GameTDB", &game.info_url);
        if ui.button("📁 Open Folder").clicked() {
            let _ = open::that(&game.path);
        }
    });
}
