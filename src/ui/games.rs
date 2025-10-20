// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::{SortBy, ViewAs},
    games,
    ui::{games_grid, games_list},
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if app.config.contents.mount_point.as_os_str().is_empty() {
            ui.label("Click on ☰ ⏵ 🖴 to choose a Drive or Directory");
            return;
        }

        ui.add_space(5.);
        update_top_bar(ui, app);
        ui.add_space(10.);

        match app.config.contents.view_as {
            ViewAs::Grid => games_grid::update(ui, app),
            ViewAs::List => games_list::update(ui, app),
        }
    });
}

fn update_top_bar(ui: &mut egui::Ui, app: &mut App) {
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
                    matches!(app.config.contents.view_as, ViewAs::Grid),
                    "🗓 Grid",
                )
                .clicked()
            {
                app.config.contents.view_as = ViewAs::Grid;
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
            }

            if ui
                .selectable_label(
                    matches!(app.config.contents.view_as, ViewAs::List),
                    "🗒 List",
                )
                .clicked()
            {
                app.config.contents.view_as = ViewAs::List;
                if let Err(e) = app.config.write() {
                    app.toasts.lock().error(e.to_string());
                }
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
