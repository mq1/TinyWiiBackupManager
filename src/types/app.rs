// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::thread;
use poll_promise::Promise;
use anyhow::{anyhow, Result};
use eframe::egui;
use eframe::egui::{FontId, RichText};
use rfd::FileDialog;
use crate::types::drive::Drive;
use crate::types::game::Game;

#[derive(Default)]
pub struct App {
    drives: Vec<Drive>,
    current_drive: Option<Drive>,
    games: Vec<(Game, bool)>,
    adding_game: Option<Promise<Result<()>>>,
}

impl App {
    pub(crate) fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            drives: Drive::list(),
            ..Default::default()
        }
    }

    fn refresh_games(&mut self) -> Result<()> {
        let drive = self.current_drive.as_ref().ok_or(anyhow!("No drive selected"))?;
        self.games = drive.get_games()?.into_iter().map(|game| (game, false)).collect();

        Ok(())
    }

    fn add_games(&mut self, ctx: &egui::Context) {
        let drive = self.current_drive.clone().unwrap();

        let files = FileDialog::new()
            .add_filter("WII Game", &["iso", "wbfs"])
            .pick_files();

        if let Some(files) = files {
            let promise = {
                let ctx = ctx.clone();
                let (sender, promise) = Promise::new();
                thread::spawn(move || {
                    for file in files {
                        // TODO: error handling
                        let _ = drive.add_game(&file);
                    }

                    sender.send(Ok(()));
                    ctx.request_repaint();
                });
                promise
            };
            self.adding_game = Some(promise);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Drives", |ui| {
                    for drive in self.drives.clone().into_iter() {
                        if ui.button(drive.to_string()).clicked() {
                            self.current_drive = Some(drive);
                            self.refresh_games().unwrap();
                        }
                    }
                });
            });
        });

        if let Some(drive) = &self.current_drive {
            egui::TopBottomPanel::bottom("drive_info").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Drive:");
                    ui.label(drive.to_string());
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current_drive.is_none() {
                ui.heading("          ⬆ Select a drive");
                return;
            }

            if let Some(promise) = &self.adding_game {
                match promise.ready() {
                    Some(Ok(())) => {
                        self.refresh_games().unwrap();
                        self.adding_game = None;
                    }
                    Some(Err(e)) => {
                        ui.heading("Error adding game");
                        ui.label(e.to_string());
                    }
                    None => {
                        ui.heading("Adding game(s)...");
                        ui.spinner();
                    }
                }
                return;
            }

            ui.heading("Games");

            ui.add_space(10.0);

            if ui.button("Add games").clicked() {
                self.add_games(ctx);
            }

            ui.add_space(10.0);

            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .column(egui_extras::Column::auto_with_initial_suggestion(1000.).resizable(true))
                .column(egui_extras::Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label(RichText::new("Game").font(FontId::proportional(16.0)));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Size").font(FontId::proportional(16.0)));
                    });
                })
                .body(|mut body| {
                    for game in self.games.iter_mut() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut game.1, game.0.display_title.clone());
                            });
                            row.col(|ui| {
                                ui.label(format!("{:.2} GiB", game.0.size as f32 / 1073741824.));
                            });
                        });
                    }
                });
        });
    }
}
