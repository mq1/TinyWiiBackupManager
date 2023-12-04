// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui::{self, FontId, RichText};
use egui_modal::{Icon, Modal};
use poll_promise::Promise;
use rfd::FileDialog;
use std::thread;

use crate::app::App;

pub fn view(ctx: &egui::Context, app: &mut App) {
    if let Some((i, total)) = *app.adding_games_progress.lock().unwrap() {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Area::new("adding_games_progress")
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.heading("Adding games");

                    ui.add_space(10.0);
                    ui.spinner();
                    ui.add_space(10.0);

                    ui.label(&format!("{}/{}", i, total));
                });
        });

        if app.games.is_some() {
            app.games = None;
        }

        return;
    }

    let drive = app.current_drive.clone().unwrap();

    let drive_cloned = drive.clone();
    let promise = app.games.get_or_insert_with(|| {
        Promise::spawn_thread("get_games", move || drive_cloned.get_games())
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading(&drive.name);

            ui.separator();

            ui.label(format!(
                "📁 {}/{} GiB",
                drive.available_space, drive.total_space
            ));
        });

        ui.add_space(10.0);

        match promise.ready_mut() {
            None => {
                ui.spinner();
            }
            Some(Err(err)) => {
                ui.label(&format!("Error: {}", err));
            }
            Some(Ok(games)) => {
                let delete_games_dialog = Modal::new(ctx, "delete_games");
                delete_games_dialog.show(|ui| {
                    delete_games_dialog.title(ui, "Delete games");
                    delete_games_dialog.frame(ui, |ui| {
                        delete_games_dialog.body_and_icon(
                            ui,
                            "Are you sure you want to delete the selected games?",
                            Icon::Warning,
                        );
                    });
                    delete_games_dialog.buttons(ui, |ui| {
                        delete_games_dialog.button(ui, "No");
                        if delete_games_dialog.caution_button(ui, "Yes").clicked() {
                            for (i, game) in games.clone().iter().enumerate() {
                                if game.checked {
                                    game.delete().unwrap();
                                    games.remove(i);
                                }
                            }
                        }
                    })
                });

                ui.horizontal(|ui| {
                    if ui.button("✅ Select all").clicked() {
                        for game in games.iter_mut() {
                            game.checked = true;
                        }
                    }

                    if ui.button("❌ Deselect all").clicked() {
                        for game in games.iter_mut() {
                            game.checked = false;
                        }
                    }

                    if ui.button("➕ Add games").clicked() {
                        let files = FileDialog::new()
                            .add_filter("WII Game", &["iso", "wbfs"])
                            .pick_files();

                        if let Some(files) = files {
                            let adding_games_progress = app.adding_games_progress.clone();
                            let drive_cloned = drive.clone();

                            thread::spawn(move || {
                                *adding_games_progress.lock().unwrap() =
                                    Some((0usize, files.len()));

                                for (i, file) in files.iter().enumerate() {
                                    drive_cloned.add_game(file).unwrap();
                                    adding_games_progress.lock().unwrap().unwrap().0 = i + 1;
                                }

                                *adding_games_progress.lock().unwrap() = None;
                            });
                        }
                    }

                    if ui.button("🗑 Delete selected").clicked() {
                        delete_games_dialog.open();
                    }
                });

                ui.separator();

                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .column(
                        egui_extras::Column::auto_with_initial_suggestion(1000.).resizable(true),
                    )
                    .column(egui_extras::Column::remainder())
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.label(RichText::new("🎮 Game").font(FontId::proportional(16.0)));
                        });
                        header.col(|ui| {
                            ui.label(RichText::new("📁 Size").font(FontId::proportional(16.0)));
                        });
                    })
                    .body(|mut body| {
                        for game in games.iter_mut() {
                            body.row(20.0, |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut game.checked, game.display_title.clone());
                                });
                                row.col(|ui| {
                                    ui.label(format!("{:.2} GiB", game.size as f32 / 1073741824.));
                                });
                            });
                        }
                    });
            }
        }
    });
}
