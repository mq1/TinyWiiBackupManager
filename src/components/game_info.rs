use crate::components::fake_link::fake_link;
use crate::game::{ConsoleType, Game};
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use anyhow::anyhow;
use eframe::egui::{self, Id, RichText};
use egui_inbox::UiInboxSender;
use size::Size;

pub fn ui_game_info_window(
    ctx: &egui::Context,
    game: &mut Game,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
) {
    let window_title = game.display_title.clone();
    let game_clone = game.clone();

    egui::Window::new(&window_title)
        .id(Id::new(game.id))
        .open(&mut game.info_opened)
        .show(ctx, |ui| {
            ui_game_info_content(ui, game_clone, sender, task_processor);
        });
}

fn ui_game_info_content(
    ui: &mut egui::Ui,
    game: Game,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("📝 Title:").strong());
        ui.label(RichText::new(&game.title));
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🆔 Game ID:").strong());
        ui.label(RichText::new(&game.id_str).monospace());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🎮 Console:").strong());
        ui.label(match game.console {
            ConsoleType::GameCube => "GameCube",
            ConsoleType::Wii => "Wii",
        });
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
            let _ = sender.send(anyhow!(e).into());
        }
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("🌐 GameTDB:").strong());
        ui.hyperlink(&game.info_url);
    });

    if let Some(info) = &game.info {
        ui.horizontal(|ui| {
            ui.label(RichText::new("🌍 Region:").strong());
            ui.label(info.region.as_ref());
        });
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("💿 Disc Metadata");
    ui.add_space(5.0);
    if let Ok(meta) = game.load_disc_meta() {
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

    if let Ok(meta) = game.load_disc_meta() {
        // if all are None, show a message
        if meta.crc32.is_none() && meta.md5.is_none() && meta.sha1.is_none() && meta.xxh64.is_none()
        {
            ui.label(
                RichText::new("No integrity info available").color(ui.visuals().warn_fg_color),
            );
        }

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

    ui.separator();
    if ui.button("🔍 Verify Disc").clicked() {
        game.spawn_verify_task(0, 1, task_processor);
    }
}
