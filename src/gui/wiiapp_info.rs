// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::gui::fake_link::fake_link;
use crate::messages::BackgroundMessage;
use crate::util::wiiapps::WiiApp;
use anyhow::anyhow;
use eframe::egui::{self, RichText};
use egui_inbox::UiInboxSender;
use size::Size;

pub fn ui_wiiapp_info_window(
    ctx: &egui::Context,
    wiiapp: &mut WiiApp,
    sender: &mut UiInboxSender<BackgroundMessage>,
) {
    let wiiapp_clone = wiiapp.clone();

    egui::Window::new(&wiiapp.meta.name)
        .open(&mut wiiapp.info_opened)
        .show(ctx, |ui| {
            ui_wiiapp_info_content(ui, wiiapp_clone, sender);
        });
}

fn ui_wiiapp_info_content(
    ui: &mut egui::Ui,
    wiiapp: WiiApp,
    sender: &mut UiInboxSender<BackgroundMessage>,
) {
    ui.add(egui::Label::new(&wiiapp.meta.long_description).wrap());
    ui.add_space(10.);
    ui.separator();

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Version:", egui_phosphor::regular::TAG)).strong());
        ui.label(&wiiapp.meta.version);
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Release Date:",
                egui_phosphor::regular::CALENDAR
            ))
            .strong(),
        );
        ui.label(&wiiapp.meta.release_date.to_string());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Coder:", egui_phosphor::regular::USER)).strong());
        ui.label(&wiiapp.meta.coder);
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!(
                "{} Size on disk:",
                egui_phosphor::regular::HARD_DRIVE
            ))
            .strong(),
        );
        ui.label(Size::from_bytes(wiiapp.size).to_string());
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} Path:", egui_phosphor::regular::FOLDER)).strong());
        if fake_link(ui, &wiiapp.path.display().to_string()).clicked()
            && let Err(e) = open::that(&wiiapp.path)
        {
            let _ = sender.send(anyhow!(e).into());
        }
    });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} View on OSCWii:", egui_phosphor::regular::LINK)).strong(),
        );
        ui.hyperlink(&wiiapp.oscwii);
    });
}
