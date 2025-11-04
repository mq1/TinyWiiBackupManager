// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("info".into());

    if app.is_info_open {
        modal.show(ctx, |ui: &mut egui::Ui| {
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

            ui.heading("💡 Projects that power TinyWiiBackupManager:");
            ui.hyperlink("https://github.com/emilk/egui");
            ui.hyperlink("https://github.com/encounter/nod");
            ui.hyperlink("https://www.gametdb.com/");
            ui.hyperlink_to("And many others", "https://github.com/mq1/TinyWiiBackupManager/blob/main/Cargo.toml");

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
                    app.is_info_open = false;
                }

                ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

                if ui.button("📁 Open Data Directory").clicked()
                    && let Err(e) = open::that(&app.data_dir)
                {
                    app.notifications.show_err(e.into());
                }

                if ui.button("🌐 Wiki").clicked()
                    && let Err(e) = open::that(env!("CARGO_PKG_HOMEPAGE"))
                {
                    app.notifications.show_err(e.into());
                }

                if ui.button(" Source Code").clicked()
                    && let Err(e) = open::that(env!("CARGO_PKG_REPOSITORY"))
                {
                    app.notifications.show_err(e.into());
                }
            })
        });
    }
}
