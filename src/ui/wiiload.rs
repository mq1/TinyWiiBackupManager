// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{ui, wiiload};
use eframe::egui;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing.y *= 2.;

        ui.heading(format!("{} Wiiload v0.5", egui_phosphor::regular::MONITOR_ARROW_UP));
        ui.add_space(10.);

        ui.label(format!("{} Wiiload is a method of loading .dol and .elf files over the network.", egui_phosphor::regular::INFO));
        ui.label(format!("{} Also, you can use Wiiload to install homebrew applications directly onto your SD card.", egui_phosphor::regular::INFO));
        ui.label(format!("{} If the icon in the very bottom right of the Homebrew Channel is lit up, it should work for you.", egui_phosphor::regular::INFO));
        ui.label(format!("{} Pressing the home button in the Homebrew Channel will reveal your Wii's IP.", egui_phosphor::regular::INFO));

        ui.separator();

        let style = ui.style();
        let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

        group.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(" Type your Wii's IP:");

                ui.add(
                    egui::TextEdit::singleline(&mut app.config.contents.wii_ip)
                        .desired_width(150.)
                        .hint_text("Wii IP"),
                );
            });
        });

        if ui
            .button(format!("{} Upload a Homebrew App (zip/dol/elf)", egui_phosphor::regular::UPLOAD))
            .clicked()
            && let Some(path) = ui::dialogs::choose_file_to_push(frame) {
                wiiload::spawn_push_file_task(app, path);
                app.save_config();
            }
    });
}
