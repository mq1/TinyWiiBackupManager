// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use crate::util;
use eframe::egui;

pub fn ui_wiiload(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.set_max_width(150.0);

        ui.label("🔢 Wii IP");
        ui.text_edit_singleline(&mut app.settings.wii_ip);

        if ui.button("⬆ Send .zip").clicked() {
            let res = rfd::FileDialog::new()
                .set_title("Select Wii App")
                .add_filter("Wii App", &["zip", "ZIP"])
                .pick_file();

            if let Some(path) = res {
                let wii_ip = app.settings.wii_ip.clone();

                app.task_processor.spawn_task(move |ui_sender| {
                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(format!(
                        "Uploading Wii App: {}",
                        path.display()
                    )));

                    let excluded_files = util::wiiload::push(&path, &wii_ip)?;

                    let mut msg = format!("Uploaded {}", path.display());
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
        }
    });
}
