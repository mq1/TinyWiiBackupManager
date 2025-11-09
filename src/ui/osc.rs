// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::ViewAs,
    ui::{osc_grid, osc_list},
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.osc_apps.is_empty() {
            ui.heading("Loading OSC Apps...");
            return;
        }

        if app.downloading_osc_icons.is_none() {
            app.download_osc_icons();
        }

        view_top_bar(ui, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => osc_grid::update(ui, app),
            ViewAs::List => osc_list::update(ui, app),
        }
    });
}

fn view_top_bar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(move |ui| {
        let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(2.);
            ui.label("🔎");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.osc_app_search)
                        .desired_width(200.)
                        .hint_text("Search by Name"),
                )
                .changed()
            {
                app.update_filtered_osc_apps();
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.config.contents.wii_ip)
                        .desired_width(100.)
                        .hint_text("Wii IP"),
                );

                ui.label("Wii IP (for Wiiload)");
            });

            ui.add_space(10.);

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    if let Err(e) = app.config.write() {
                        app.notifications.show_err(e);
                    }
                }

                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    if let Err(e) = app.config.write() {
                        app.notifications.show_err(e);
                    }
                }
            });
        });
    });
}
