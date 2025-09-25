// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::base_dir::BaseDir;
use crate::gui::devs::get_developer_icon;
use crate::gui::wiiapp_info::ui_wiiapp_info_window;
use crate::messages::BackgroundMessage;
use crate::task::TaskProcessor;
use crate::util::wiiapps::WiiApp;
use eframe::egui::{self, Image, RichText};
use egui_inbox::UiInboxSender;
use size::Size;

const MIN_CARD_WIDTH: f32 = 150.0;
const CARD_HEIGHT: f32 = 150.0;
const CARD_PADDING: f32 = 5.0;

pub fn ui_wiiapp_grid(ui: &mut egui::Ui, app: &mut App) {
    let wiiapps = &mut app.wiiapps;

    if !wiiapps.is_empty()
        && let Some(base_dir) = &app.base_dir
    {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();
            ui.set_width(available_width);

            // We don't want to divide by zero
            let num_columns = std::cmp::max(
                1,
                (available_width / (MIN_CARD_WIDTH + CARD_PADDING * 2.)) as usize,
            );

            // Calculate the width of each card
            let card_width = (available_width / num_columns as f32) - CARD_PADDING * 4.5;

            egui::Grid::new("app_grid")
                .spacing(egui::Vec2::splat(CARD_PADDING * 2.))
                .show(ui, |ui| {
                    for row in wiiapps.chunks_mut(num_columns) {
                        for wiiapp in row {
                            ui_wiiapp_card(
                                ui,
                                card_width,
                                &mut app.inbox.sender(),
                                wiiapp,
                                base_dir,
                                &app.task_processor,
                            );
                            ui_wiiapp_info_window(ui.ctx(), wiiapp, &mut app.inbox.sender());
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

fn ui_wiiapp_card(
    ui: &mut egui::Ui,
    card_width: f32,
    sender: &mut UiInboxSender<BackgroundMessage>,
    wiiapp: &mut WiiApp,
    base_dir: &BaseDir,
    task_processor: &TaskProcessor,
) {
    let card = egui::Frame::group(ui.style()).corner_radius(5.0);
    card.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(card_width);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} {}",
                    egui_phosphor::regular::TAG,
                    &wiiapp.meta.version
                ));

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(Size::from_bytes(wiiapp.size).to_string());
                });
            });

            // Centered content
            ui.vertical_centered_justified(|ui| {
                let image = Image::from_uri(&wiiapp.icon_uri)
                    .max_width(128.0)
                    .maintain_aspect_ratio(true)
                    .show_loading_spinner(true);
                ui.add(image);

                ui.add_space(5.);

                ui.add(egui::Label::new(RichText::new(&wiiapp.meta.name).strong()).truncate());

                let avatar = get_developer_icon(&wiiapp.meta.coder);
                ui.add(
                    egui::Label::new(format!("{} by {}", avatar, &wiiapp.meta.coder)).truncate(),
                );
            });

            // Actions
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    // Info button
                    if ui
                        .button(egui_phosphor::regular::INFO)
                        .on_hover_text("Show App Info")
                        .clicked()
                    {
                        wiiapp.toggle_info();
                    }

                    // Remove button
                    if ui
                        .button(egui_phosphor::regular::TRASH)
                        .on_hover_text("Remove App")
                        .clicked()
                        && let Err(e) = wiiapp.remove() {
                            let _ = sender.send(e.into());
                        }

                    // Update button
                    if let Some(oscwii) = &wiiapp.oscwii_app
                        && oscwii.version != wiiapp.meta.version
                    {
                        if ui
                            .add(
                                egui::Button::new(format!(
                                    "{} {}",
                                    egui_phosphor::regular::ARROW_UP,
                                    oscwii.version
                                ))
                                .min_size(egui::vec2(ui.available_width(), 0.0)),
                            )
                            .clicked()
                        {
                            let app_name = wiiapp.meta.name.clone();
                            let zip_url = oscwii.assets.archive.url.clone();
                            let base_dir = base_dir.clone();

                            task_processor.spawn_task(move |ui_sender| {
                                let _ = ui_sender.send(BackgroundMessage::UpdateStatus(format!(
                                    "Updating {app_name}",
                                )));

                                base_dir.add_zip_from_url(&zip_url)?;

                                let _ = ui_sender
                                    .send(BackgroundMessage::Info(format!("Updated {app_name}")));

                                let _ = ui_sender.send(BackgroundMessage::DirectoryChanged);

                                Ok(())
                            });
                        };
                    } else {
                        ui.add_enabled(
                            false,
                            egui::Button::new("Up to date")
                                .min_size(egui::vec2(ui.available_width(), 0.0)),
                        );
                    }
                });
            });
        });
    });
}
