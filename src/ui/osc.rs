// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    config::ViewAs,
    ui::{UiAction, osc_grid, osc_list},
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    egui::CentralPanel::default().show(ctx, |ui| {
        if app_state.osc_apps.is_empty() {
            ui.heading("Loading OSC Apps...");
            return;
        }

        if app_state.downloading_osc_icons.is_none() {
            ui_buffers.action = Some(UiAction::TriggerDownloadOscIcons);
        }

        update_top_bar(ui, app_state, ui_buffers);
        ui.add_space(10.);

        match ui_buffers.config.contents.view_as {
            ViewAs::Grid => osc_grid::update(ui, app_state, ui_buffers),
            ViewAs::List => osc_list::update(ui, app_state, ui_buffers),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, _app_state: &AppState, ui_buffers: &mut UiBuffers) {
    ui.horizontal(move |ui| {
        let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
        group.show(ui, |ui| {
            ui.set_height(21.);
            ui.add_space(2.);
            ui.label("🔎");

            if ui
                .add(
                    egui::TextEdit::singleline(&mut ui_buffers.osc_apps_filter)
                        .desired_width(200.)
                        .hint_text("Search by Name"),
                )
                .changed()
            {
                ui_buffers.action = Some(UiAction::ApplyFilterOscApps);
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut ui_buffers.config.contents.wii_ip)
                        .desired_width(100.)
                        .hint_text("Wii IP"),
                );

                ui.label(" 📮  Wii IP (for Wiiload)");
            });

            ui.add_space(10.);

            let group = egui::Frame::group(ui.style()).fill(ui.style().visuals.extreme_bg_color);
            group.show(ui, |ui| {
                if ui
                    .selectable_label(ui_buffers.config.contents.view_as == ViewAs::List, "☰")
                    .on_hover_text("View as List")
                    .clicked()
                {
                    ui_buffers.config.contents.view_as = ViewAs::List;
                    ui_buffers.save_config();
                }

                if ui
                    .selectable_label(ui_buffers.config.contents.view_as == ViewAs::Grid, "")
                    .on_hover_text("View as Grid")
                    .clicked()
                {
                    ui_buffers.config.contents.view_as = ViewAs::Grid;
                    ui_buffers.save_config();
                }
            });
        });
    });
}
