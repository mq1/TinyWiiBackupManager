use crate::game::Game;
use eframe::egui::{self, CentralPanel, RichText, ScrollArea, TopBottomPanel};
use size::Size;

pub fn ui_game_info_modal(ctx: &egui::Context, game: &Game, open: &mut bool) {
    if !*open {
        return;
    }

    let mut modal = egui::Modal::new(game.id.clone().into());
    modal.area = modal.area.default_size([700., 550.]);

    modal.show(ctx, |ui| {
        TopBottomPanel::top("game_info_title_panel").show_inside(ui, |ui| {
            ui.heading(&game.display_title);
            ui.add_space(2.0);
        });

        TopBottomPanel::bottom("game_info_actions_panel").show_inside(ui, |ui| {
            ui.add_space(7.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    *open = false;
                }
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui_game_info_content(ui, game);
            });
        });
    });
}

fn ui_game_info_content(ui: &mut egui::Ui, game: &Game) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("📝 Title:").strong());
        ui.label(RichText::new(&game.display_title));
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🆔 Game ID:").strong());
        ui.label(RichText::new(&game.id).monospace());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🎮 Console:").strong());
        ui.label(if game.is_gc { "GameCube" } else { "Wii" });
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
        if ui.hyperlink(game.path.to_string_lossy().as_ref()).clicked() {
            let _ = open::that(&game.path);
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
    if let Some(ref meta) = game.disc_meta {
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

    if let Some(ref meta) = game.disc_meta {
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
            ui.label("No integrity info available");
        }
    } else {
        ui.label(RichText::new("Couldn't read disc metadata").color(ui.visuals().warn_fg_color));
    }
}
