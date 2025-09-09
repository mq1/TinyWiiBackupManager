// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use eframe::egui;
use eframe::egui::RichText;

pub fn ui_oscwii_window(ctx: &egui::Context, app: &mut App) {
    egui::Window::new("🏪 Open Shop Channel")
        .open(&mut app.oscwii_window_open)
        .auto_sized()
        .collapsible(false)
        .movable(false)
        .show(ctx, |ui| {
            ui.set_width(ctx.screen_rect().width() - 14.);
            ui.set_height(ctx.screen_rect().height() - 69.); // nice

            ui.horizontal(|ui| {
                ui.label("Filter 🔎");

                let edit = egui::TextEdit::singleline(&mut app.oscwii_filter)
                    .desired_width(ui.available_width());
                ui.add(edit);
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

                            let _ = ui.button(format!("⬆ {}", wiiapp.version));

                            ui.end_row();
                        }
                    });
            });
        });
}
