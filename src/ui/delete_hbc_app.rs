// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;
use std::fs;

pub fn update(ctx: &egui::Context, app: &mut App, hbc_app_i: u16) {
    let modal = egui::Modal::new("delete_hbc_app".into());
    let mut action = Action::None;

    let hbc_app = &app.hbc_apps[hbc_app_i as usize];
    let text = format!("Are you sure you want to delete {}?", &hbc_app.meta.name);

    modal.show(ctx, |ui| {
        ui.heading("⚠ Delete HBC App");

        ui.add_space(10.);

        ui.label(text);

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🗑 Delete").clicked() {
                action = Action::Delete;
            }

            if ui.button("❌ Cancel").clicked() {
                action = Action::Cancel;
            }
        });
    });

    match action {
        Action::None => {}
        Action::Delete => {
            if let Err(e) = fs::remove_dir_all(&hbc_app.path) {
                app.notifications.show_err(e.into());
            }

            app.current_modal = ui::Modal::None;
            app.refresh_hbc_apps(ctx);
        }
        Action::Cancel => {
            app.current_modal = ui::Modal::None;
        }
    }
}

enum Action {
    None,
    Delete,
    Cancel,
}
