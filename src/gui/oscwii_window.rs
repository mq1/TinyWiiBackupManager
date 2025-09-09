// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use crate::util;
use eframe::egui;
use eframe::egui::{RichText, TextEdit};

pub fn ui_oscwii_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("🏪 Open Shop Channel")
        .open(&mut app.oscwii_window_open)
        .auto_sized()
        .collapsible(false)
        .movable(false)
        .show(ctx, |ui| {
            ui.set_width(ctx.screen_rect().width() - 14.);
            ui.set_height(ctx.screen_rect().height() - 69.5);

            ui.horizontal(|ui| {
                ui.label("🔎 Filter");
                ui.add(TextEdit::singleline(&mut app.oscwii_filter).hint_text("Type something"));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.set_max_width(150.0);
                    ui.text_edit_singleline(&mut app.settings.wii_ip);
                    ui.label("🔢 Wii IP");
                });
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                egui::Grid::new("oscwii_apps")
                    .striped(true)
                    .start_row(1)
                    .show(ui, |ui| {
                        ui.label(RichText::new("⭐ App").strong());
                        ui.label(RichText::new("👸 Author").strong());
                        ui.label(RichText::new("📥 Download").strong());
                        ui.label(RichText::new("📮 Wiiload").strong());
                        ui.end_row();

                        let filter = app.oscwii_filter.to_lowercase();
                        for wiiapp in app.oscwii_apps.apps.iter().filter(|wiiapp| {
                            wiiapp.name.to_lowercase().contains(&filter)
                                || wiiapp.slug.to_lowercase().contains(&filter)
                        }) {
                            ui.hyperlink_to(
                                &wiiapp.name,
                                format!("https://oscwii.org/library/app/{}", wiiapp.slug),
                            )
                            .on_hover_text(&wiiapp.description.short);

                            ui.label(&wiiapp.author);

                            if let Some(base_dir) = &app.base_dir
                                && ui.button(format!("⬇ {}", wiiapp.version)).clicked()
                            {
                                let zip_url = &wiiapp.assets.archive.url;
                                let base_dir = base_dir.clone();
                                let zip_url = zip_url.clone();
                                let wiiapp_name = wiiapp.name.clone();

                                app.task_processor.spawn_task(move |ui_sender| {
                                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
                                        format!("Downloading {wiiapp_name}"),
                                    ));

                                    base_dir.add_zip_from_url(&zip_url)?;

                                    let _ = ui_sender.send(BackgroundMessage::Info(format!(
                                        "Downloaded {wiiapp_name}"
                                    )));

                                    Ok(())
                                });
                            }

                            if ui.button(format!("⬆ {}", wiiapp.version)).clicked() {
                                let wii_ip = app.settings.wii_ip.clone();
                                let url = wiiapp.assets.archive.url.clone();
                                let wiiapp_name = wiiapp.name.clone();

                                app.task_processor.spawn_task(move |ui_sender| {
                                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
                                        format!("Uploading {wiiapp_name}"),
                                    ));

                                    util::wiiload::push_url(&url, &wii_ip)?;

                                    let _ = ui_sender.send(BackgroundMessage::Info(format!(
                                        "Uploaded {wiiapp_name}"
                                    )));

                                    Ok(())
                                });
                            };

                            ui.end_row();
                        }
                    });
            });
        });
}
