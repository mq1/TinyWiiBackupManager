// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    ui::{self, UiAction, View},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &AppState, ui_buffers: &mut UiBuffers) {
    let frame =
        egui::Frame::side_top_panel(&ctx.style()).fill(ctx.style().visuals.extreme_bg_color);

    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(57.)
        .frame(frame)
        .show(ctx, |ui| {
            ui.add_space(6.);

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::Games,
                        egui::RichText::new("🎮").size(26.),
                    ),
                )
                .on_hover_text("View your Wii games")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::Games));
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::HbcApps,
                        egui::RichText::new("★").size(26.),
                    ),
                )
                .on_hover_text("View your HBC apps")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::HbcApps));
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::Osc,
                        egui::RichText::new("🏪").size(26.),
                    ),
                )
                .on_hover_text("Open Shop Channel")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::Osc));
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::Wiiload,
                        egui::RichText::new("📮").size(26.),
                    ),
                )
                .on_hover_text("Wiiload")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::Wiiload));
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::Tools,
                        egui::RichText::new("🔧").size(26.),
                    ),
                )
                .on_hover_text("Tools")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::Tools));
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == View::Settings,
                        egui::RichText::new("⛭").size(26.),
                    ),
                )
                .on_hover_text("Settings")
                .clicked()
            {
                ui_buffers.action = Some(UiAction::OpenView(View::Settings));
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(6.);

                if ui
                    .add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new("ℹ").size(26.)),
                    )
                    .on_hover_text(format!("{} Info", env!("CARGO_PKG_NAME")))
                    .clicked()
                {
                    ui_buffers.action = Some(UiAction::OpenModal(ui::Modal::Info));
                }

                if ui
                    .add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new("🖴").size(26.)),
                    )
                    .on_hover_text("Select Drive/Mount Point")
                    .clicked()
                {
                    ui_buffers.choose_mount_point.pick_directory();
                }

                if let Some(update_info) = &app.update_info
                    && ui
                        .add_sized(
                            Vec2::splat(40.),
                            egui::Button::new(egui::RichText::new("❕").size(26.)),
                        )
                        .on_hover_text(&update_info.ui_text)
                        .clicked()
                {
                    ui_buffers.action = Some(UiAction::OpenUpdateUrl);
                }
            });
        });
}
