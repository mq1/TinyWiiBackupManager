// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use crate::util::wiiapps::WiiApp;
use eframe::egui::{self, Image, RichText};
use egui_inbox::UiInboxSender;
use size::Size;

const CARD_WIDTH: f32 = 188.5;
const CARD_HEIGHT: f32 = 220.0;
const GRID_SPACING: f32 = 10.0;

pub fn ui_app_grid(ui: &mut egui::Ui, app: &mut App) {
    let wiiapps = &mut app.wiiapps;

    if !wiiapps.is_empty() {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_min_width(ui.available_width());

            let num_columns =
                (ui.available_width() / (CARD_WIDTH + GRID_SPACING / 2.)).max(1.) as usize;

            egui::Grid::new("app_grid")
                .min_col_width(CARD_WIDTH)
                .min_row_height(CARD_HEIGHT)
                .spacing(egui::Vec2::splat(GRID_SPACING))
                .show(ui, |ui| {
                    for row in wiiapps.chunks_mut(num_columns) {
                        for wiiapp in row {
                            ui_app_card(ui, &mut app.inbox.sender(), &app.task_processor, wiiapp);
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

fn ui_app_card(
    ui: &mut egui::Ui,
    sender: &mut UiInboxSender<BackgroundMessage>,
    task_processor: &TaskProcessor,
    wiiapp: &mut WiiApp,
) {
    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Size::from_bytes(wiiapp.size).to_string());
                });
            });

            // Centered content
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                let image = Image::from_bytes("placeholder", &[])
                    .max_height(128.0)
                    .maintain_aspect_ratio(true)
                    .show_loading_spinner(true);
                ui.add(image);

                ui.add_space(5.);

                let label = egui::Label::new(RichText::new(&wiiapp.name).strong()).truncate();
                ui.add(label);
            });

            // Actions
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(32.);

                    // Info button
                    if ui.button("ℹ").on_hover_text("Show Game Info").clicked() {}

                    // Archive button
                    if ui
                        .button("📦")
                        .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                        .clicked()
                    {}

                    // Integrity check button
                    if ui.button("🔎").on_hover_text("Integrity Check").clicked() {}

                    // Remove button
                    if ui.button("🗑").on_hover_text("Remove Game").clicked() {}
                });
            });
        });
    });
}
