// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
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
                        app.current_view == ui::View::Games,
                        egui::RichText::new("🎮").size(26.),
                    ),
                )
                .on_hover_text("View your Wii games")
                .clicked()
            {
                app.change_view(ctx, ui::View::Games);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == ui::View::HbcApps,
                        egui::RichText::new("★").size(26.),
                    ),
                )
                .on_hover_text("View your HBC apps")
                .clicked()
            {
                app.change_view(ctx, ui::View::HbcApps);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == ui::View::Osc,
                        egui::RichText::new("🏪").size(26.),
                    ),
                )
                .on_hover_text("Open Shop Channel")
                .clicked()
            {
                app.change_view(ctx, ui::View::Osc);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == ui::View::Wiiload,
                        egui::RichText::new("📮").size(26.),
                    ),
                )
                .on_hover_text("Wiiload")
                .clicked()
            {
                app.change_view(ctx, ui::View::Wiiload);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == ui::View::Tools,
                        egui::RichText::new("🔧").size(26.),
                    ),
                )
                .on_hover_text("Tools")
                .clicked()
            {
                app.change_view(ctx, ui::View::Tools);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        app.current_view == ui::View::Settings,
                        egui::RichText::new("⛭").size(26.),
                    ),
                )
                .on_hover_text("Settings")
                .clicked()
            {
                app.change_view(ctx, ui::View::Settings);
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
                    app.current_modal = ui::Modal::Info;
                }

                if ui
                    .add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new("🖴").size(26.)),
                    )
                    .on_hover_text("Select Drive/Mount Point")
                    .clicked()
                {
                    app.choose_mount_point.pick_directory();
                }

                if let Some(update_info) = &app.update_info
                    && ui
                        .add_sized(
                            Vec2::splat(40.),
                            egui::Button::new(egui::RichText::new("❕").size(26.)),
                        )
                        .on_hover_text(update_info.to_string())
                        .clicked()
                    && let Err(e) = update_info.open_url()
                {
                    app.notifications.show_err(e);
                }
            });
        });
}
