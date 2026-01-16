// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::extensions::SUPPORTED_DISC_EXTENSIONS;
use crate::ui;
use eframe::egui;
use egui_phosphor::regular as ph;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing.y *= 2.;

        ui.heading(format!(
            "{} Nintendo Optical Disc format conversion",
            ph::FLOW_ARROW
        ));

        let style = ui.style();
        let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

        group.show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!("{} Input file", ph::SIGN_IN));

            ui.horizontal(|ui| {
                if ui
                    .button(ph::FILE_MAGNIFYING_GLASS)
                    .on_hover_text("Choose input file")
                    .clicked()
                    && let Some(path) = ui::dialogs::choose_input_disc_path(frame)
                {
                    app.nod_gui_input_path = path.to_string_lossy().to_string();
                }

                ui.add(
                    egui::widgets::TextEdit::singleline(&mut app.nod_gui_input_path)
                        .hint_text("Input file path")
                        .min_size(egui::vec2(ui.available_width(), 0.0)),
                );
            });
        });

        group.show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!("{} Output file", ph::SIGN_OUT));

            ui.label(format!(
                "{} Supported extensions: {}",
                ph::INFO,
                SUPPORTED_DISC_EXTENSIONS.join(", ")
            ));

            ui.horizontal(|ui| {
                if ui
                    .button(ph::FILE_MAGNIFYING_GLASS)
                    .on_hover_text("Choose output file")
                    .clicked()
                    && let Some(path) = ui::dialogs::choose_output_disc_path(frame)
                {
                    app.nod_gui_output_path = path.to_string_lossy().to_string();
                }

                ui.add(
                    egui::widgets::TextEdit::singleline(&mut app.nod_gui_output_path)
                        .hint_text("Output file path")
                        .min_size(egui::vec2(ui.available_width(), 0.0)),
                );
            });
        });

        group.show(ui, |ui| {
            ui.set_width(ui.available_width());

            ui.label(format!("{} Output Options", ph::DISC));

            if ui
                .checkbox(
                    &mut app.config.contents.scrub_update_partition,
                    format!("{} Remove Update Partition on WBFS/CISO", ph::FILE_DASHED),
                )
                .changed()
            {
                app.save_config();
            }

            if ui
                .checkbox(
                    &mut app.config.contents.always_split,
                    format!("{} Split WBFS/ISO to 4GB-32KB", ph::ARROWS_SPLIT),
                )
                .changed()
            {
                app.save_config();
            }
        });

        if ui
            .button(format!("{} Run conversion", ph::ARROW_FAT_RIGHT))
            .clicked()
        {
            app.run_single_conversion(frame);
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{} Using NOD under the hood:", ph::HEART));
                ui.hyperlink_to(
                    format!("{} https://github.com/encounter/nod", ph::GITHUB_LOGO),
                    "https://github.com/encounter/nod",
                );
            });
        });
    });
}
