// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{convert, disc_info::DiscInfo};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &App, discs: &[DiscInfo]) {
    egui::Modal::new("confirm_conversion".into()).show(ctx, |ui: &mut egui::Ui| {
        ui.heading(format!(
            "{} {} Games selected for conversion",
            egui_phosphor::regular::SWORD,
            discs.len()
        ));
        ui.label("(Existing games are automatically ignored)");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                for info in discs {
                    ui.label(format!(
                        "{} {} [{}]",
                        egui_phosphor::regular::CARET_RIGHT,
                        &info.title,
                        info.id.as_str()
                    ));
                }
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui
                .button(format!(
                    "{} Start conversion",
                    egui_phosphor::regular::ARROW_FAT_RIGHT
                ))
                .clicked()
            {
                convert::spawn_add_games_task(app, discs.into());
                app.send_msg(Message::CloseModal);
            }

            if ui
                .button(format!("{} Cancel", egui_phosphor::regular::X))
                .clicked()
            {
                app.send_msg(Message::CloseModal);
            }
        })
    });
}
