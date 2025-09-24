// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use crate::util;
use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};

pub fn ui_oscwii(ui: &mut egui::Ui, app: &mut App) {
    ui.group(|ui| {
        ui.heading(format!(
            "{} Open Shop Channel",
            egui_phosphor::regular::STOREFRONT
        ));
        ui.separator();

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} Filter",
                egui_phosphor::regular::MAGNIFYING_GLASS
            ));
            ui.add(TextEdit::singleline(&mut app.oscwii_filter).hint_text("Type something"));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.set_max_width(150.0);
                ui.text_edit_singleline(&mut app.settings.wii_ip);
                ui.label(format!("{} Wii IP", egui_phosphor::regular::WIFI_HIGH));
            });
        });

        ui.separator();

        let filter = app.oscwii_filter.to_lowercase();
        let filtered_apps = app
            .oscwii_apps
            .apps
            .iter()
            .filter(|wiiapp| {
                wiiapp.name.to_lowercase().contains(&filter)
                    || wiiapp.slug.to_lowercase().contains(&filter)
            })
            .collect::<Vec<_>>();

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(150.))
            .column(Column::exact(100.))
            .column(Column::exact(100.))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading(format!("{} App", egui_phosphor::regular::STAR));
                });
                header.col(|ui| {
                    ui.heading(format!("{} Version", egui_phosphor::regular::TAG));
                });
                header.col(|ui| {
                    ui.heading(format!("{} Author", egui_phosphor::regular::USER));
                });
                header.col(|ui| {
                    ui.heading(format!("{} Actions", egui_phosphor::regular::PACKAGE));
                });
            })
            .body(|body| {
                body.rows(20., filtered_apps.len(), |mut row| {
                    let wiiapp = filtered_apps[row.index()];

                    // Cell for the app name
                    row.col(|ui| {
                        ui.hyperlink_to(
                            &wiiapp.name,
                            format!("https://oscwii.org/library/app/{}", wiiapp.slug),
                        )
                            .on_hover_text(&wiiapp.description.short);
                    });

                    // Cell for the version
                    row.col(|ui| {
                        ui.label(&wiiapp.version);
                    });

                    // Cell for the author
                    row.col(|ui| {
                        ui.label(&wiiapp.author);
                    });

                    // Cell for download/upload buttons
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            if let Some(base_dir) = &app.base_dir {
                                if ui.button(format!("{} Download", egui_phosphor::regular::ARROW_DOWN)).clicked() {
                                    let zip_url = &wiiapp.assets.archive.url;
                                    let base_dir = base_dir.clone();
                                    let zip_url = zip_url.clone();
                                    let wiiapp_name = wiiapp.name.clone();

                                    app.task_processor.spawn_task(move |ui_sender| {
                                        let _ =
                                            ui_sender.send(BackgroundMessage::UpdateStatus(
                                                format!("Downloading {wiiapp_name}"),
                                            ));
                                        base_dir.add_zip_from_url(&zip_url)?;
                                        let _ = ui_sender.send(BackgroundMessage::Info(
                                            format!("Downloaded {wiiapp_name}"),
                                        ));
                                        Ok(())
                                    });
                                }
                            } else {
                                ui.add_enabled(false, egui::Button::new("Base dir not set"));
                            }

                            if ui.button(format!("{} Wiiload", egui_phosphor::regular::ARROW_UP)).clicked() {
                                let wii_ip = app.settings.wii_ip.clone();
                                let url = wiiapp.assets.archive.url.clone();
                                let wiiapp_name = wiiapp.name.clone();

                                app.task_processor.spawn_task(move |ui_sender| {
                                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
                                        format!("Uploading {wiiapp_name}"),
                                    ));

                                    let excluded_files =
                                        util::wiiload::push_url(&url, &wii_ip)?;

                                    let mut msg = format!("Uploaded {wiiapp_name}");
                                    if !excluded_files.is_empty() {
                                        msg += "\n\nThe following files may need to be manually transferred:";
                                        for file in excluded_files {
                                            msg += &format!("\n• {file}");
                                        }
                                    }

                                    let _ = ui_sender.send(BackgroundMessage::Info(msg));
                                    Ok(())
                                });
                            }
                        });
                    });
                });
            });
    });
}
