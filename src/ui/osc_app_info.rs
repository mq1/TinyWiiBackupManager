// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui::developers::get_developer_emoji;
use eframe::egui;
use eframe::egui::OpenUrl;
use egui_phosphor::regular as ph;
use itertools::Itertools;

pub fn update(ctx: &egui::Context, app: &mut App, osc_app_i: u16) {
    egui::Modal::new("osc_app_info".into()).show(ctx, |ui: &mut egui::Ui| {
        let osc_app = &app.osc_apps[osc_app_i as usize];

        ui.set_height(ctx.available_rect().height() - 80.);
        ui.set_width(700.);

        ui.heading(&osc_app.meta.name);
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!(
                "{} Author: {}",
                get_developer_emoji(&osc_app.meta.author),
                &osc_app.meta.author
            ));

            ui.label(format!(
                "{} Authors: {}",
                ph::USERS_THREE,
                osc_app.meta.authors.join(", ")
            ));
            ui.label(format!(
                "{} Category: {}",
                ph::TAG_SIMPLE,
                &osc_app.meta.category
            ));
            ui.label(format!(
                "{} Contributors: {}",
                ph::USERS_FOUR,
                osc_app.meta.contributors.join(", ")
            ));
            ui.label(format!(
                "{} Downloads: {}",
                ph::CLOUD_ARROW_DOWN,
                osc_app.meta.downloads
            ));
            ui.label(format!(
                "{} Flags: {}",
                ph::FLAG,
                osc_app.meta.flags.iter().map(|f| f.as_str()).join(", ")
            ));
            ui.label(format!(
                "{} Package Type: {}",
                ph::PACKAGE,
                osc_app.meta.package_type.as_str()
            ));
            ui.label(format!(
                "{} Peripherals: {}",
                ph::USB,
                &osc_app
                    .meta
                    .peripherals
                    .iter()
                    .map(|p| p.as_str())
                    .join(", ")
            ));
            ui.label(format!(
                "{} Release Date: {}",
                ph::CALENDAR,
                osc_app.meta.release_date.date()
            ));
            ui.label(format!(
                "{} Subdirectories: {}",
                ph::FOLDER_OPEN,
                osc_app.meta.subdirectories.join(", ")
            ));
            ui.label(format!(
                "{} Supported Platforms: {}",
                ph::DEVICES,
                osc_app
                    .meta
                    .supported_platforms
                    .iter()
                    .map(|p| p.as_str())
                    .join(", ")
            ));
            ui.label(format!(
                "{} Uncompressed size: {}",
                ph::SCALES,
                &osc_app.meta.uncompressed_size
            ));
            ui.label(format!("{} Version: {}", ph::TAG, &osc_app.meta.version));
            ui.label(format!(
                "{} Short Description: {}",
                ph::CLIPBOARD_TEXT,
                &osc_app.meta.description.short
            ));

            ui.separator();

            ui.label(&osc_app.meta.description.long);
        });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui.button(format!("{} Close", ph::X)).clicked() {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui
                .button(format!("{} Open oscwii.org page", ph::LINK))
                .clicked()
            {
                let osc_app = &app.osc_apps[osc_app_i as usize];
                let url = format!("https://oscwii.org/library/app/{}", &osc_app.meta.slug);
                ctx.open_url(OpenUrl::new_tab(&url));
            }
        });
    });
}
