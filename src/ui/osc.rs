// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::{
    config::ViewAs,
    ui::{osc_grid, osc_list},
};
use eframe::egui;
use egui_phosphor::fill as ph;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.osc_apps.is_empty() {
            ui.heading(format!("{} Loading OSC Apps...", ph::CLOUD_ARROW_DOWN));
            return;
        }

        if !app.has_osc_icons_downlading_started {
            app.download_osc_icons();
        }

        update_top_bar(ui, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => osc_grid::update(ui, app),
            ViewAs::List => osc_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, app: &mut App) {
    let current_view_as = app.config.contents.view_as;

    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    ui.horizontal(|ui| {
        group.show(ui, |ui| {
            ui.add_space(3.);
            ui.vertical(|ui| {
                ui.add_space(2.);
                ui.label(ph::MAGNIFYING_GLASS);
            });

            if ui
                .add(
                    egui::TextEdit::singleline(&mut app.osc_apps_filter)
                        .desired_width(200.)
                        .hint_text("Search by Name"),
                )
                .changed()
            {
                app.update_filtered_osc_apps();
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            group.show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut app.config.contents.wii_ip)
                        .desired_width(100.)
                        .hint_text("Wii IP"),
                );

                ui.label(format!("  {}  Wii IP (for Wiiload)", ph::WIFI_HIGH));
            });

            ui.add_space(10.);

            group.show(ui, |ui| {
                if ui
                    .selectable_label(current_view_as == ViewAs::List, ph::ROWS)
                    .on_hover_text("View as List")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::List;
                    app.save_config();
                }

                if ui
                    .selectable_label(current_view_as == ViewAs::Grid, ph::SQUARES_FOUR)
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    app.config.contents.view_as = ViewAs::Grid;
                    app.save_config();
                }
            });
        });
    });
}
