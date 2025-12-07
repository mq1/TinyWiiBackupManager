// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui::developers::get_developer_emoji;
use eframe::egui;
use eframe::egui::OpenUrl;
use itertools::Itertools;

pub fn update(ctx: &egui::Context, app: &mut App, osc_app_i: u16) {
    egui::Modal::new("osc_app_info".into()).show(ctx, |ui: &mut egui::Ui| {
        let osc_app = &app.osc_apps[osc_app_i as usize];

        egui::ScrollArea::vertical()
            .max_height(500.)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(&osc_app.meta.name);

                ui.separator();

                ui.label(format!(
                    "{} Author: {}",
                    get_developer_emoji(&osc_app.meta.author),
                    &osc_app.meta.author
                ));

                ui.label(format!(
                    "{} Authors: {}",
                    egui_phosphor::regular::USERS_THREE,
                    osc_app.meta.authors.join(", ")
                ));
                ui.label(format!(
                    "{} Category: {}",
                    egui_phosphor::regular::TAG_SIMPLE,
                    &osc_app.meta.category
                ));
                ui.label(format!(
                    "{} Contributors: {}",
                    egui_phosphor::regular::USERS_FOUR,
                    osc_app.meta.contributors.join(", ")
                ));
                ui.label(format!(
                    "{} Downloads: {}",
                    egui_phosphor::regular::CLOUD_ARROW_DOWN,
                    osc_app.meta.downloads
                ));
                ui.label(format!(
                    "{} Flags: {}",
                    egui_phosphor::regular::FLAG,
                    osc_app.meta.flags.iter().map(|f| f.as_str()).join(", ")
                ));
                ui.label(format!(
                    "{} Package Type: {}",
                    egui_phosphor::regular::PACKAGE,
                    osc_app.meta.package_type.as_str()
                ));
                ui.label(format!(
                    "{} Peripherals: {}",
                    egui_phosphor::regular::USB,
                    &osc_app
                        .meta
                        .peripherals
                        .iter()
                        .map(|p| p.as_str())
                        .join(", ")
                ));
                ui.label(format!(
                    "{} Release Date: {}",
                    egui_phosphor::regular::CALENDAR,
                    osc_app.meta.release_date.date()
                ));
                ui.label(format!(
                    "{} Subdirectories: {}",
                    egui_phosphor::regular::FOLDER_OPEN,
                    osc_app.meta.subdirectories.join(", ")
                ));
                ui.label(format!(
                    "{} Supported Platforms: {}",
                    egui_phosphor::regular::DEVICES,
                    osc_app
                        .meta
                        .supported_platforms
                        .iter()
                        .map(|p| p.as_str())
                        .join(", ")
                ));
                ui.label(format!(
                    "{} Uncompressed size: {}",
                    egui_phosphor::regular::SCALES,
                    &osc_app.meta.uncompressed_size
                ));
                ui.label(format!(
                    "{} Version: {}",
                    egui_phosphor::regular::TAG,
                    &osc_app.meta.version
                ));
                ui.label(format!(
                    "{} Short Description: {}",
                    egui_phosphor::regular::CLIPBOARD_TEXT,
                    &osc_app.meta.description.short
                ));

                ui.separator();

                ui.label(&osc_app.meta.description.long);
            });

        ui.add_space(10.);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui
                .button(format!("{} Close", egui_phosphor::regular::X))
                .clicked()
            {
                app.close_modal();
            }

            ui.add_sized(egui::Vec2::new(1., 21.), egui::Separator::default());

            if ui
                .button(format!(
                    "{} Open oscwii.org page",
                    egui_phosphor::regular::LINK
                ))
                .clicked()
            {
                let osc_app = &app.osc_apps[osc_app_i as usize];
                let url = format!("https://oscwii.org/library/app/{}", &osc_app.meta.slug);
                ctx.open_url(OpenUrl::new_tab(&url));
            }
        });
    });
}
