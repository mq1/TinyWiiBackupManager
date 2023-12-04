// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui;
use poll_promise::Promise;

use crate::app::App;
use crate::pages::Page;
use crate::types::drive::Drive;

pub fn view(ctx: &egui::Context, app: &mut App) {
    let promise = app
        .drives
        .get_or_insert_with(|| Promise::spawn_thread("get_drives", || Drive::get_drives()));

    egui::CentralPanel::default().show(ctx, |_ui| {
        egui::Area::new("drives_area")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.heading("Choose a drive");

                ui.add_space(10.0);

                match promise.ready() {
                    None => {
                        ui.spinner();
                    }
                    Some(drives) => {
                        ui.horizontal(|ui| {
                            let selected_text = if let Some(drive) = &app.current_drive {
                                &drive.name
                            } else {
                                "No drive selected"
                            };

                            egui::ComboBox::from_id_source("drives")
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for drive in drives {
                                        ui.selectable_value(
                                            &mut app.current_drive,
                                            Some(drive.clone()),
                                            &drive.name,
                                        );
                                    }
                                });

                            if ui.button("Open").clicked() && app.current_drive.is_some() {
                                app.page = Page::Games;
                            }
                        });
                    }
                }
            });
    });
}
