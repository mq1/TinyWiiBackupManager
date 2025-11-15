// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    convert,
    disc_info::DiscInfo,
    ui::UiAction,
};
use eframe::egui;

pub fn update(
    ctx: &egui::Context,
    app_state: &AppState,
    ui_buffers: &mut UiBuffers,
    discs: &[DiscInfo],
) {
    egui::Modal::new("confirm_conversion".into()).show(ctx, |ui: &mut egui::Ui| {
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
                let task_processor = &app_state.task_processor;
                let config_contents = &app_state.config.contents;
                convert::spawn_add_games_task(task_processor, config_contents, discs.into());

                ui_buffers.action = Some(UiAction::CloseModal)
            }

            if ui.button("❌ Cancel").clicked() {
                ui_buffers.action = Some(UiAction::CloseModal)
            }
        })
    });
}
