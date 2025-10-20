// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, covers, ui, util, wiitdb};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::SidePanel::left("nav")
        .resizable(false)
        .exact_width(57.)
        .show(&ctx, |ui| {
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
                app.current_view = ui::View::Games;
                app.update_title(ctx);
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
                app.current_view = ui::View::HbcApps;
                app.update_title(ctx);
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
                app.current_view = ui::View::Osc;
                app.update_title(ctx);
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
                app.current_view = ui::View::Settings;
                app.update_title(ctx);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add_space(6.);

                if ui
                    .add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new("ℹ").size(26.)),
                    )
                    .on_hover_text("Wiki")
                    .clicked()
                {
                    let _ = open::that(env!("CARGO_PKG_HOMEPAGE"));
                }

                egui::Popup::menu(
                    &ui.add_sized(
                        Vec2::splat(40.),
                        egui::Button::new(egui::RichText::new("☰").size(26.)),
                    )
                    .on_hover_text("Additional actions"),
                )
                .show(|ui| {
                    if ui
                        .button(egui::RichText::new("🖴 Choose a Drive/Directory").size(15.))
                        .clicked()
                    {
                        app.choose_mount_point.pick_directory();
                    }

                    ui.separator();

                    if ui
                        .button(egui::RichText::new("📥 Download wiitdb.xml").size(15.))
                        .clicked()
                    {
                        wiitdb::spawn_download_task(app);
                    }

                    ui.separator();

                    if ui
                        .button(egui::RichText::new("📥 Download all covers").size(15.))
                        .clicked()
                    {
                        covers::spawn_download_all_covers_task(app);
                    }

                    if cfg!(target_os = "macos") {
                        ui.separator();
                        if ui
                            .button(egui::RichText::new("👻 Run dot_clean").size(15.))
                            .clicked()
                        {
                            if let Err(e) = util::run_dot_clean(&app.config.contents.mount_point) {
                                app.toasts.error(e.to_string());
                            }
                        }
                    }
                });

                if let Some(update_info) = &app.update_info {
                    if ui
                        .add_sized(
                            Vec2::splat(40.),
                            egui::Button::new(egui::RichText::new("🔔").size(26.)),
                        )
                        .on_hover_text(update_info.to_string())
                        .clicked()
                    {
                        if let Err(e) = update_info.open_url() {
                            app.toasts.error(e.to_string());
                        }
                    }
                }
            });
        });
}
