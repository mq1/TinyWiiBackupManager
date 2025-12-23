// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::ui::{self, View};
use eframe::egui::{self, OpenUrl, Vec2};
use egui_phosphor::regular as ph;

const UPDATE_URL: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/releases/latest");

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    let current_view = app.current_view;

    let style = &ctx.style();
    let nav_frame = egui::Frame::side_top_panel(style).fill(style.visuals.extreme_bg_color);

    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(57.)
        .frame(nav_frame)
        .show(ctx, |ui| {
            ui.add_space(6.);

            if app.show_zoom_buttons {
                ui.horizontal(|ui| {
                    let spacing = ui.spacing_mut();
                    spacing.button_padding = egui::vec2(3., 0.);
                    spacing.item_spacing = egui::vec2(4., 0.);

                    if ui.button(ph::MINUS).clicked() {
                        egui::gui_zoom::zoom_out(ctx);
                    }
                    if ui.button(ph::PLUS).clicked() {
                        egui::gui_zoom::zoom_in(ctx);
                    }
                });

                ui.separator();
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Games,
                        egui::RichText::new(ph::SWORD).size(26.),
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
                        egui::RichText::new(ph::WAVES).size(26.),
                    ),
                )
                .on_hover_text("View your Homebrew Channel apps")
                .clicked()
            {
                app.open_view(ctx, View::HbcApps);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Osc,
                        egui::RichText::new(ph::STOREFRONT).size(26.),
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
                        egui::RichText::new(ph::MONITOR_ARROW_UP).size(26.),
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
                        current_view == View::NodGui,
                        egui::RichText::new(ph::FLOW_ARROW).size(26.),
                    ),
                )
                .on_hover_text("Nintendo Optical Disc format conversion")
                .clicked()
            {
                app.open_view(ctx, View::NodGui);
            }

            if ui
                .add_sized(
                    Vec2::splat(40.),
                    egui::Button::selectable(
                        current_view == View::Tools,
                        egui::RichText::new(ph::WRENCH).size(26.),
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
                        egui::RichText::new(ph::GEAR).size(26.),
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
                        egui::Button::new(egui::RichText::new(ph::INFO).size(26.)),
                    )
                    .on_hover_text(format!("{} Info", env!("CARGO_PKG_NAME")))
                    .clicked()
                {
                    app.send_msg(Message::OpenModal(ui::Modal::Info));
                }

                if ui
                    .add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new(ph::HARD_DRIVE).size(26.)),
                    )
                    .on_hover_text("Select Drive/Mount Point")
                    .clicked()
                    && let Some(path) = ui::dialogs::choose_mount_point(frame)
                {
                    app.update_mount_point(ctx, path);
                }

                if let Some(version) = &app.update
                    && ui
                        .add_sized(
                            Vec2::splat(40.),
                            egui::Button::new(egui::RichText::new(ph::SEAL_WARNING).size(26.)),
                        )
                        .on_hover_text(format!(
                            "{} A new version is available: {}",
                            ph::CLOUD_CHECK,
                            &version
                        ))
                        .clicked()
                {
                    ctx.open_url(OpenUrl::new_tab(UPDATE_URL));
                }
            });
        });
}
