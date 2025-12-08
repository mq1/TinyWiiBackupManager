// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::txtcodes::TxtCodesSource;
use crate::ui::accent::AccentColor;
use eframe::egui;
use eframe::egui::{ThemePreference, Vec2};
use nod::common::Format;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(ctx, |ui| {
        update_top_bar(ui, ctx, app);
        ui.add_space(10.);

        ui.style_mut().spacing.item_spacing.y *= 2.;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let style = ui.style();
            let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Wii Output Format",
                    egui_phosphor::regular::DISC
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.wii_output_format,
                        Format::Wbfs,
                        "WBFS (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.wii_output_format,
                        Format::Iso,
                        "ISO (very large)",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} GameCube Output Format",
                    egui_phosphor::regular::DISC
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.gc_output_format,
                        Format::Iso,
                        "ISO (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.gc_output_format,
                        Format::Ciso,
                        "CISO (small but poor compatibility)",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Archive Output Format",
                    egui_phosphor::regular::FILE_ARCHIVE
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.archive_format,
                        Format::Rvz,
                        "RVZ [zstd -19] (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.archive_format,
                        Format::Iso,
                        "ISO (very large)",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Split Output",
                    egui_phosphor::regular::ARROWS_SPLIT
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.always_split,
                        false,
                        "Only when needed (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.always_split,
                        true,
                        "Always 4GB-32KB",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Remove Update Partition on WBFS/CISO",
                    egui_phosphor::regular::FILE_DASHED
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.scrub_update_partition,
                        false,
                        "No (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.scrub_update_partition,
                        true,
                        "Yes (saves some space)",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Delete sources when adding games",
                    egui_phosphor::regular::TRASH
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.remove_sources_games,
                        false,
                        "No (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(&mut app.config.contents.remove_sources_games, true, "Yes")
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Delete sources when adding apps",
                    egui_phosphor::regular::TRASH
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.remove_sources_apps,
                        false,
                        "No (recommended)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(&mut app.config.contents.remove_sources_apps, true, "Yes")
                    .changed()
                {
                    app.save_config();
                }
            });

            group.show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.heading(format!(
                    "{} Cheat Codes source",
                    egui_phosphor::regular::FILE_TXT
                ));

                if ui
                    .radio_value(
                        &mut app.config.contents.txt_codes_source,
                        TxtCodesSource::WebArchive,
                        "geckocodes.org (web.archive.org)    (Recommended, high quality)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.txt_codes_source,
                        TxtCodesSource::Rc24,
                        "codes.rc24.xyz    (Lower quality)",
                    )
                    .changed()
                {
                    app.save_config();
                }

                if ui
                    .radio_value(
                        &mut app.config.contents.txt_codes_source,
                        TxtCodesSource::GameHacking,
                        "gamehacking.org    (Up to date, download might fail)",
                    )
                    .changed()
                {
                    app.save_config();
                }
            });
        });
    });
}

fn update_top_bar(ui: &mut egui::Ui, ctx: &egui::Context, app: &mut App) {
    let style = ui.style();
    let group = egui::Frame::group(style).fill(style.visuals.extreme_bg_color);

    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        group.show(ui, |ui| {
            ui.style_mut().spacing.item_spacing.x = 0.;

            update_theme_btn(
                ui,
                ctx,
                app,
                ThemePreference::Dark,
                "Dark Theme",
                egui_phosphor::regular::MOON,
            );
            update_theme_btn(
                ui,
                ctx,
                app,
                ThemePreference::Light,
                "Light Theme",
                egui_phosphor::regular::SUN,
            );
            update_theme_btn(
                ui,
                ctx,
                app,
                ThemePreference::System,
                "System Theme",
                egui_phosphor::regular::CIRCLE_HALF,
            );
        });

        group.show(ui, |ui| {
            ui.style_mut().spacing.item_spacing.x = 0.;

            update_accent_btn(ui, ctx, app, AccentColor::Gray);
            update_accent_btn(ui, ctx, app, AccentColor::Green);
            update_accent_btn(ui, ctx, app, AccentColor::Yellow);
            update_accent_btn(ui, ctx, app, AccentColor::Orange);
            update_accent_btn(ui, ctx, app, AccentColor::Red);
            update_accent_btn(ui, ctx, app, AccentColor::Pink);
            update_accent_btn(ui, ctx, app, AccentColor::Purple);
            update_accent_btn(ui, ctx, app, AccentColor::Blue);

            if cfg!(target_os = "macos") {
                update_accent_btn(ui, ctx, app, AccentColor::System);
            }

            ui.add_space(5.);
            ui.add_sized(
                Vec2::splat(21.),
                egui::Label::new(egui_phosphor::regular::PALETTE),
            );
        });

        ui.shrink_height_to_current();
    });
}

fn update_theme_btn(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut App,
    theme_preference: ThemePreference,
    text: &str,
    emoji: &str,
) {
    if ui
        .selectable_label(
            app.config.contents.theme_preference == theme_preference,
            emoji,
        )
        .on_hover_text(text)
        .clicked()
    {
        app.config.contents.theme_preference = theme_preference;
        ctx.set_theme(theme_preference);
        app.save_config();
    }
}

fn update_accent_btn(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    app: &mut App,
    accent_color: AccentColor,
) {
    let color = egui::Color32::from(accent_color);
    let text = egui::RichText::new(egui_phosphor::regular::SPIRAL).color(color.to_opaque());

    if ui
        .selectable_label(app.config.contents.accent_color == accent_color, text)
        .on_hover_text(accent_color.as_str())
        .clicked()
    {
        ctx.all_styles_mut(|style| {
            style.visuals.selection.bg_fill = color;
        });

        app.config.contents.accent_color = accent_color;
        app.save_config();
    }
}
