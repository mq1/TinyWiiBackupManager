// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui::developers::get_developer_emoji;
use eframe::egui;
use egui_phosphor::fill as ph;

pub fn update(ctx: &egui::Context, app: &mut App, hbc_app_i: u16) {
    egui::Modal::new("hbc_app_info".into()).show(ctx, |ui: &mut egui::Ui| {
        let hbc_app = &app.hbc_apps[hbc_app_i as usize];

        ui.set_height(ctx.available_rect().height() - 80.);
        ui.set_width(700.);

        ui.heading(&hbc_app.meta.name);
        ui.label(format!("{} Path: {}", ph::FOLDER, hbc_app.get_path_str()));

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!(
                "{} Coder: {}",
                get_developer_emoji(&hbc_app.meta.coder),
                &hbc_app.meta.coder
            ));
            ui.label(format!(
                "{} Version: {}",
                ph::PUSH_PIN,
                &hbc_app.meta.version
            ));
            ui.label(format!(
                "{} Release Date: {}",
                ph::CALENDAR,
                &hbc_app.meta.release_date
            ));
            ui.label(format!(
                "{} Short Description: {}",
                ph::CLIPBOARD_TEXT,
                &hbc_app.meta.short_description
            ));

            ui.separator();

            ui.set_width(ui.available_width());
            ui.label(&hbc_app.meta.long_description);
        });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button(format!("{} Close", ph::X_CIRCLE)).clicked() {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui
                .button(format!("{} Open Directory", ph::FOLDER))
                .clicked()
            {
                app.open_hbc_app_dir(hbc_app_i);
            }
        })
    });
}
