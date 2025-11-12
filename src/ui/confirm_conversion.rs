// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, convert, ui};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    let modal = egui::Modal::new("confirm_conversion".into());
    let mut action = Action::None;

    let discs = &app.discs_to_convert;

    modal.show(ctx, |ui: &mut egui::Ui| {
        ui.heading(format!("🎮 {} Games selected for conversion", discs.len()));
        ui.label("(Existing games are automatically ignored)");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                for info in discs {
                    ui.label(format!(
                        "⏵ {} [{}]",
                        info.header.game_title_str(),
                        info.header.game_id_str()
                    ));
                }
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button("✅ Start conversion").clicked() {
                action = Action::StartConversion;
            }

            if ui.button("❌ Cancel").clicked() {
                action = Action::Cancel;
            }
        })
    });

    match action {
        Action::None => {}
        Action::StartConversion => {
            convert::spawn_add_games_task(app, discs.clone());
            app.current_modal = ui::Modal::None;
        }
        Action::Cancel => {
            app.current_modal = ui::Modal::None;
        }
    }
}

enum Action {
    None,
    StartConversion,
    Cancel,
}
