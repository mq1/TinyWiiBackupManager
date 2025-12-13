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
                ui.label(format!("{} Version {}", egui_phosphor::regular::PUSH_PIN, env!("CARGO_PKG_VERSION")));
                ui.label(format!("© 2025 Manuel Quarneti | {} GPL-3.0-only", egui_phosphor::regular::FILE_TEXT));
            });
        });

        ui.separator();

        ui.label(format!("{} TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo. Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited.", egui_phosphor::regular::WARNING));

        ui.separator();
        ui.add_space(10.);

        ui.heading(format!("{} Need help modding your Wii?", egui_phosphor::regular::INFO));
        ui.hyperlink_to(format!("{} Wii Hacks Guide", egui_phosphor::regular::LINK), "https://wii.hacks.guide/");

        ui.add_space(10.);

        ui.heading(format!("{} 3rd party libraries and licenses", egui_phosphor::regular::LIGHTBULB));
        ui.hyperlink_to(format!("{} List", egui_phosphor::regular::LINK), "https://github.com/mq1/TinyWiiBackupManager/wiki/3rd-party-libraries-and-licenses");

        ui.add_space(10.);

        ui.heading(format!("{} Special thanks to", egui_phosphor::regular::HEART));
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
            if ui.button(format!("{} Close", egui_phosphor::regular::X)).clicked() {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui.button(format!("{} Open Data Directory", egui_phosphor::regular::FOLDER)).clicked() {
                app.open_data_dir();
            }

            if ui.button(format!("{} Wiki", egui_phosphor::regular::LINK)).clicked() {
                ctx.open_url(OpenUrl::new_tab(env!("CARGO_PKG_HOMEPAGE")));
            }

            if ui.button(format!("{} Source Code", egui_phosphor::regular::GITHUB_LOGO)).clicked() {
                ctx.open_url(OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
            }
        })
    });
}
