// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::ui::{self, View};
use eframe::egui::{self, OpenUrl, Vec2};

const UPDATE_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/latest");

pub fn update(ctx: &egui::Context, app: &mut App) {
    let current_view = app.current_view;

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
                        current_view == View::Games,
                        egui::RichText::new("🎮").size(26.),
                    ),
                )
                .on_hover_text("View your Wii games")
                .clicked()
            {
                app.open_view(ctx, View::Games);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::HbcApps,
                        egui::RichText::new("★").size(26.),
                    ),
                )
                .on_hover_text("View your HBC apps")
                .clicked()
            {
                app.open_view(ctx, View::HbcApps);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Osc,
                        egui::RichText::new("🏪").size(26.),
                    ),
                )
                .on_hover_text("Open Shop Channel")
                .clicked()
            {
                app.open_view(ctx, View::Osc);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Wiiload,
                        egui::RichText::new("📮").size(26.),
                    ),
                )
                .on_hover_text("Wiiload")
                .clicked()
            {
                app.open_view(ctx, View::Wiiload);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Tools,
                        egui::RichText::new("🔧").size(26.),
                    ),
                )
                .on_hover_text("Tools")
                .clicked()
            {
                app.open_view(ctx, View::Tools);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Settings,
                        egui::RichText::new("⛭").size(26.),
                    ),
                )
                .on_hover_text("Settings")
                .clicked()
            {
                app.open_view(ctx, View::Settings);
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
                    app.send_msg(Message::OpenModal(ui::Modal::Info));
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

                if let Some(version) = &app.update
                    && ui
                        .add_sized(
                            Vec2::splat(40.),
                            egui::Button::new(egui::RichText::new("❕").size(26.)),
                        )
                        .on_hover_text(format!("A new version is available: {}", &version))
                        .clicked()
                {
                    ctx.open_url(OpenUrl::new_tab(UPDATE_URL));
                }
            });
        });
}
