// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    ui::{self, developers::get_developer_emoji},
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App, hbc_app_i: u16) {
    let modal = egui::Modal::new("hbc_app_info".into());
    let mut action = Action::None;

    let hbc_app = &app.hbc_apps[hbc_app_i as usize];

    modal.show(ctx, |ui: &mut egui::Ui| {
        ui.heading(&hbc_app.meta.name);

        ui.separator();

        // Path
        ui.label("📁 Path: ".to_string() + hbc_app.path.to_str().unwrap_or("Unknown"));

        ui.separator();

        ui.label(format!(
            "{} Coder: {}",
            get_developer_emoji(&hbc_app.meta.coder),
            &hbc_app.meta.coder
        ));
        ui.label("📌 Version: ".to_string() + &hbc_app.meta.version);
        ui.label("📆 Release Date: ".to_string() + &hbc_app.meta.release_date);
        ui.label("📄 Short Description: ".to_string() + &hbc_app.meta.short_description);

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(400.)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(&hbc_app.meta.long_description);
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button("❌ Close").clicked() {
                action = Action::Close;
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui.button("📁 Open Directory").clicked() {
                action = Action::OpenDirectory;
            }
        })
    });

    match action {
        Action::None => {}
        Action::OpenDirectory => {
            if let Err(e) = open::that(&hbc_app.path) {
                app.notifications.show_err(e.into());
            }
        }
        Action::Close => {
            app.current_modal = ui::Modal::None;
        }
    }
}

enum Action {
    None,
    OpenDirectory,
    Close,
}
