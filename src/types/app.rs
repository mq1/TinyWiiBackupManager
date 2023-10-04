// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::fmt::format;
use std::sync::Mutex;
use std::thread;

use anyhow::{anyhow, Result};
use eframe::egui;
use eframe::egui::{FontId, ProgressBar, RichText};
use once_cell::sync::Lazy;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

use crate::types::drive::Drive;
use crate::types::game::Game;

static ADDING_GAMES: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static ADDING_GAMES_PROGRESS: Lazy<Mutex<(usize, usize)>> = Lazy::new(|| Mutex::new((0, 0)));
static ADDING_GAMES_RESULT: Lazy<Mutex<Option<Result<()>>>> = Lazy::new(|| Mutex::new(None));


#[derive(Default)]
pub struct App {
    drives: Vec<Drive>,
    current_drive: Option<Drive>,
    games: Vec<(Game, bool)>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
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

    fn delete_games(&mut self) {
        let res = MessageDialog::new().set_level(MessageLevel::Warning).set_title("Delete games").set_description("Are you sure you want to delete the selected games?").set_buttons(MessageButtons::YesNo).show();

        if res == MessageDialogResult::Yes {
            let games = self.games.iter().filter(|game| game.1).map(|game| game.0.clone()).collect::<Vec<_>>();

            for game in games {
                game.delete().unwrap();
            }
        }

        self.refresh_games().unwrap();
    }

    fn add_games(&mut self) {
        let drive = self.current_drive.clone().unwrap();

        let files = FileDialog::new().add_filter("WII Game", &["iso", "wbfs"]).pick_files();

        if let Some(files) = files {
            thread::spawn(move || {
                *ADDING_GAMES.lock().unwrap() = true;
                *ADDING_GAMES_RESULT.lock().unwrap() = None;

                for (i, file) in files.iter().enumerate() {
                    *ADDING_GAMES_PROGRESS.lock().unwrap() = (i + 1, files.len());
                    if let Err(e) = drive.add_game(file) {
                        *ADDING_GAMES_RESULT.lock().unwrap() = Some(Err(e));
                        return;
                    }
                }

                *ADDING_GAMES_RESULT.lock().unwrap() = Some(Ok(()));
            });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("🗄 File", |ui| {
                    if ui.button("About").clicked() {
                        let desc = format!("v{}\n{}\n\nCopyright (c) 2023 {}\n{} Licensed", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_AUTHORS"), env!("CARGO_PKG_LICENSE"));
                        MessageDialog::new().set_title(env!("CARGO_PKG_NAME")).set_description(desc).set_buttons(MessageButtons::Ok).show();
                    }

                    if ui.button("Check for updates").clicked() {
                        thread::spawn(|| {
                            let _ = crate::updater::check_for_updates();
                        });
                    }

                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("🖴 Drives", |ui| {
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
                ui.add_space(3.);
                ui.label(format!("🖴 {drive}"));
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.current_drive.is_none() {
                ui.heading("                ⬆ Select a drive");
                return;
            }

            if *ADDING_GAMES.lock().unwrap() {
                match &*ADDING_GAMES_RESULT.lock().unwrap() {
                    Some(Ok(_)) => {
                        *ADDING_GAMES.lock().unwrap() = false;
                        self.refresh_games().unwrap();
                    }
                    Some(Err(e)) => {
                        ui.heading("Error adding games");
                        ui.label(e.to_string());
                    }
                    None => {
                        let adding_games_progress = *ADDING_GAMES_PROGRESS.lock().unwrap();
                        ui.heading(format!("Adding games ({}/{})", adding_games_progress.0, adding_games_progress.1));

                        ui.add_space(10.0);

                        let progress_bar = ProgressBar::new(adding_games_progress.0 as f32 / adding_games_progress.1 as f32);
                        ui.add(progress_bar);
                    }
                }
                return;
            }

            ui.heading("🎮 Games");

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("🗑 Delete selected").clicked() {
                    self.delete_games();
                }

                if ui.button("➕ Add games").clicked() {
                    self.add_games();
                }
            });

            ui.separator();

            egui_extras::TableBuilder::new(ui).striped(true).column(egui_extras::Column::auto_with_initial_suggestion(1000.).resizable(true)).column(egui_extras::Column::remainder()).header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("🎮 Game").font(FontId::proportional(16.0)));
                });
                header.col(|ui| {
                    ui.label(RichText::new("📁 Size").font(FontId::proportional(16.0)));
                });
            }).body(|mut body| {
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
