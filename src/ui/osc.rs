// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::ViewAs,
    ui::{osc_grid, osc_list},
};
use eframe::egui::{self, Vec2};

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        view_top_bar(ui, app);
        ui.add_space(10.);

        if app.osc_apps.is_none() {
            ui.heading("Click on ⟳ to fetch the OSC Apps");
            return;
        }

        match app.config.contents.view_as {
            ViewAs::Grid => osc_grid::update(ui, app),
            ViewAs::List => osc_list::update(ui, app),
        }
    });
}

fn view_top_bar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(move |ui| {
        ui.group(|ui| {
            ui.label(egui::RichText::new("🔎").size(15.5));

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
            if ui
                .add_sized(
                    Vec2::splat(32.),
                    egui::Button::new(egui::RichText::new("⟳").size(18.)),
                )
                .on_hover_text("Fetch Apps")
                .clicked()
            {
                app.refresh_osc_apps();
            }

            ui.add_space(10.);

            ui.group(|ui| {
                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    if let Err(e) = app.config.write() {
                        app.toasts.error(e.to_string());
                    }
                }

                if ui
                    .selectable_label(app.config.contents.view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    if let Err(e) = app.config.write() {
                        app.toasts.error(e.to_string());
                    }
                }
            });
        });
    });
}
