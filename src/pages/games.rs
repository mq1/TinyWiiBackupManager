// SPDX-FileCopyrightText: 2024 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui::{self, FontId, RichText};
use poll_promise::Promise;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult};
use std::thread;

use crate::app::App;
use crate::pages::Page::AddingGames;

pub fn view(ctx: &egui::Context, app: &mut App) {
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
                            app.page = AddingGames;

                            let adding_games_progress = app.adding_games_progress.clone();
                            let drive_cloned = drive.clone();

                            thread::spawn(move || {
                                *adding_games_progress.lock().unwrap() =
                                    Some((0usize, files.len()));

                                for (i, file) in files.iter().enumerate() {
                                    adding_games_progress.lock().unwrap().unwrap().0 = i + 1;
                                    drive_cloned.add_game(file).unwrap();
                                }

                                *adding_games_progress.lock().unwrap() = None;
                            });
                        }
                    }

                    if ui.button("🗑 Delete selected").clicked() {
                        let res = MessageDialog::new()
                            .set_title("Delete games")
                            .set_description("Are you sure you want to delete the selected games?")
                            .set_buttons(MessageButtons::YesNo)
                            .show();

                        if res == MessageDialogResult::Yes {
                            for (i, game) in games.clone().iter().enumerate() {
                                if game.checked {
                                    game.delete().unwrap();
                                    games.remove(i);
                                }
                            }
                        }
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
                            body.row(60., |mut row| {
                                row.col(|ui| {
                                    ui.horizontal_centered(|ui| {
                                        ui.checkbox(&mut game.checked, "");

                                        let img = egui::Image::from_uri(format!(
                                            "https://art.gametdb.com/wii/cover/EN/{}.png",
                                            game.id
                                        ))
                                        .fit_to_exact_size(egui::vec2(40., 56.));
                                        ui.add(img);

                                        ui.label(game.display_title.clone());
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal_centered(|ui| {
                                        ui.label(format!(
                                            "{:.2} GiB",
                                            game.size as f32 / 1073741824.
                                        ));
                                    });
                                });
                            });
                        }
                    });
            }
        }
    });
}
