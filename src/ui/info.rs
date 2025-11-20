// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui;
use eframe::egui;
use eframe::egui::OpenUrl;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::Modal::new("info".into()).show(ctx, |ui: &mut egui::Ui| {
        ui.horizontal(|ui| {
            ui.set_height(68.);

            ui.add(
                egui::Image::from_bytes("bytes://info", ui::LOGO_BYTES)
                    .max_size(egui::Vec2::splat(64.)),
            );

            ui.vertical(|ui| {
                ui.add_space(4.);
                ui.heading(env!("CARGO_PKG_NAME"));
                ui.label(format!("📌 Version {}", env!("CARGO_PKG_VERSION")));
                ui.label("© 2025 Manuel Quarneti | 📃 GPL-3.0-only");
            });
        });

        ui.separator();

        ui.label("‼ TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo. Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited.");

        ui.separator();
        ui.add_space(10.);

        ui.heading("ℹ Need help modding your Wii?");
        ui.hyperlink_to("🌐 Wii Hacks Guide", "https://wii.hacks.guide/");

        ui.add_space(10.);

        ui.heading("💡 Projects powering TinyWiiBackupManager:");
        ui.hyperlink_to("🌐 https://github.com/emilk/egui", "https://github.com/emilk/egui");
        ui.hyperlink_to("🌐 https://github.com/encounter/nod", "https://github.com/encounter/nod");
        ui.hyperlink_to("🌐 https://www.gametdb.com/", "https://www.gametdb.com/");
        ui.hyperlink_to("💡 And many more", "https://github.com/mq1/TinyWiiBackupManager/blob/main/Cargo.toml");

        ui.add_space(10.);

        ui.heading("👏 Special thanks to");
        ui.horizontal(|ui| {
            ui.hyperlink_to("Luke Street", "https://github.com/encounter");
            ui.label("for developing nod and helping TWBM leverage it effectively.");
        });
        ui.horizontal(|ui| {
            ui.hyperlink_to("blackb0x", "https://github.com/wiidev");
            ui.label("for invaluable feedback and advice during TWBM's development.");
        });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button("❌ Close").clicked() {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui.button("📁 Open Data Directory").clicked() {
                app.open_data_dir();
            }

            if ui.button("🌐 Wiki").clicked() {
                ctx.open_url(OpenUrl::new_tab(env!("CARGO_PKG_HOMEPAGE")));
            }

            if ui.button(" Source Code").clicked() {
                ctx.open_url(OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
            }
        })
    });
}
