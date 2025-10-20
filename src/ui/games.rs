// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::SortBy,
    disc_info::DiscInfo,
    games::{self, Game},
};
use eframe::egui::{self, Margin, Vec2};

const CARD_WIDTH: f32 = 153.5;
const CARD_HEIGHT: f32 = 185.;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.label("Click on ☰ ⏵ 🖴 to choose a Drive or Directory");
            return;
        }

        ui.add_space(5.);
        view_top_bar(ui, app);
        ui.add_space(10.);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let available_width = ui.available_width();
            ui.set_width(available_width);
            let cols = (available_width / (CARD_WIDTH + 28.)).floor() as usize;

            egui::Grid::new("games")
                .num_columns(cols)
                .spacing(Vec2::splat(8.))
                .show(ui, |ui| {
                    for row in app.filtered_games.lock().chunks(cols) {
                        for game in row {
                            view_game_card(ui, game, &mut app.removing_game, &mut app.disc_info);
                        }

                        ui.end_row();
                    }
                });
        });
    });
}

fn view_top_bar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(move |ui| {
        ui.add_space(5.);
        ui.label("🔎");

        if ui
            .add(egui::TextEdit::singleline(&mut app.game_search).hint_text("Search by Title/ID"))
            .changed()
        {
            app.update_filtered_games();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(5.);

            if ui.button("✚ Add Games").clicked() {}

            if ui.button("⟳").on_hover_text("Refresh Games").clicked() {
                games::spawn_get_games_task(app);
            }

            ui.separator();

            if ui
                .selectable_label(
                    matches!(
                        app.config.contents.sort_by,
                        SortBy::SizeAscending | SortBy::SizeDescending
                    ),
                    if app.config.contents.sort_by == SortBy::SizeDescending {
                        "⚖⏷"
                    } else {
                        "⚖⏶"
                    },
                )
                .on_hover_text("Sort by size")
                .clicked()
            {
                app.config.contents.sort_by =
                    if app.config.contents.sort_by == SortBy::SizeAscending {
                        SortBy::SizeDescending
                    } else {
                        SortBy::SizeAscending
                    };
                app.apply_sorting();
            }

            if ui
                .selectable_label(
                    matches!(
                        app.config.contents.sort_by,
                        SortBy::NameAscending | SortBy::NameDescending
                    ),
                    if app.config.contents.sort_by == SortBy::NameDescending {
                        "🗛⏷"
                    } else {
                        "🗛⏶"
                    },
                )
                .on_hover_text("Sort by name")
                .clicked()
            {
                app.config.contents.sort_by =
                    if app.config.contents.sort_by == SortBy::NameAscending {
                        SortBy::NameDescending
                    } else {
                        SortBy::NameAscending
                    };
                app.apply_sorting();
            }

            ui.separator();

            if ui
                .checkbox(&mut app.show_gc, "🎲")
                .on_hover_text("Show GameCube")
                .changed()
            {
                app.update_filtered_games();
            }

            ui.separator();

            if ui
                .checkbox(&mut app.show_wii, "🎾")
                .on_hover_text("Show Wii")
                .changed()
            {
                app.update_filtered_games();
            }
        });
    });
}

fn view_game_card(
    ui: &mut egui::Ui,
    game: &Game,
    removing_game: &mut Option<Game>,
    disc_info: &mut Option<DiscInfo>,
) {
    let card = egui::Frame::group(ui.style())
        .corner_radius(10.0)
        .inner_margin(Margin::same(10));

    card.show(ui, |ui| {
        ui.set_height(CARD_HEIGHT);
        ui.set_width(CARD_WIDTH);

        ui.vertical_centered(|ui| {
            // Top row with console label on the left and size label on the right
            ui.horizontal(|ui| {
                // Console label on the left
                ui.label(if game.is_wii { "🎾 Wii" } else { "🎲 GC" });

                // Size label on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(game.size.to_string());
                });
            });

            ui.add_space(10.);

            // Middle row with image and title
            ui.add(egui::Image::new(&game.image_uri).max_height(96.0));

            ui.add_space(10.);

            ui.add(egui::Label::new(&game.display_title).truncate());

            ui.add_space(10.);

            // Bottom row with buttons

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Remove button
                if ui.button("🗑").on_hover_text("Remove Game").clicked() {
                    *removing_game = Some(game.clone());
                }

                // Integrity check button
                if ui.button("☑").on_hover_text("Integrity Check").clicked() {}

                // Archive button
                if ui
                    .button("📦")
                    .on_hover_text("Archive Game to a zstd-19 compressed RVZ")
                    .clicked()
                {}

                // Info button
                if ui
                    .add(
                        egui::Button::new("ℹ Info").min_size(egui::vec2(ui.available_width(), 0.0)),
                    )
                    .on_hover_text("Show Game Information")
                    .clicked()
                {
                    *disc_info = Some(DiscInfo::from_game_dir(&game.path).unwrap_or_default());
                }
            });
        });
    });
}
