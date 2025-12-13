// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{
    hbc_apps::{self},
    ui::{self},
};
use eframe::egui;

const CARD_WIDTH: f32 = 161.5;
const CARD_HORIZONTAL_SPACE: usize = 181;
const CARD_HEIGHT: f32 = 140.;

pub fn update(ui: &mut egui::Ui, app: &mut App) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let available_width = ui.available_width();
        ui.set_width(available_width);
        let cols = available_width as usize / CARD_HORIZONTAL_SPACE;

        ui.heading(format!(
            "{} Homebrew Channel Apps: {} found ({})",
            egui_phosphor::regular::WAVES,
            app.filtered_hbc_apps.len(),
            &app.filtered_hbc_apps_size
        ));

        ui.add_space(5.);

        for row in app.filtered_hbc_apps.chunks(cols) {
            ui.horizontal_top(|ui| {
                for hbc_app_i in row.iter().copied() {
                    update_hbc_app_card(ui, app, hbc_app_i);
                }
            });

            ui.add_space(5.);
        }
    });
}

fn update_hbc_app_card(ui: &mut egui::Ui, app: &App, hbc_app_i: u16) {
    let hbc_app = &app.hbc_apps[hbc_app_i as usize];

    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    group.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with version on the left and size label on the right
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{}  {}",
                    egui_phosphor::regular::PUSH_PIN,
                    hbc_app
                        .meta
                        .version
                        .get(..10)
                        .unwrap_or(&hbc_app.meta.version)
                ));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(hbc_app.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and name
            ui.add(egui::Image::new(&hbc_app.image_uri).max_height(48.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(egui::RichText::new(&hbc_app.meta.name).strong()).truncate());

            ui.add_space(10.);

            // Bottom row with buttons
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button
                if ui
                    .button(egui_phosphor::regular::TRASH)
                    .on_hover_text("Delete HBC App")
                    .clicked()
                {
                    app.send_msg(Message::DeleteHbcApp(hbc_app_i));
                }

                // Update button
                if let Some(osc_app_i) = hbc_app.osc_app_i {
                    let osc_app = &app.osc_apps[osc_app_i as usize];

                    if osc_app.meta.version != hbc_app.meta.version
                        && ui
                            .button(egui_phosphor::regular::CLOUD_ARROW_DOWN)
                            .on_hover_text(format!(
                                "Download update from OSC: v{}",
                                &osc_app.meta.version
                            ))
                            .clicked()
                    {
                        hbc_apps::spawn_install_app_from_url_task(
                            app,
                            osc_app.meta.assets.archive.url.clone(),
                        );
                    }
                }

                // Info button
                if ui
                    .add(
                        egui::Button::new(format!("{} Info", egui_phosphor::regular::INFO))
                            .min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show App Information")
                    .clicked()
                {
                    app.send_msg(Message::OpenModal(ui::Modal::HbcAppInfo(hbc_app_i)));
                }
            });
        });
    });
}
