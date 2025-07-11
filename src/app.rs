// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use crate::game::Game;

pub struct App {
    wbfs_dir: PathBuf,
    games: Vec<Game>,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
        };
        app.refresh_games();

        app
    }

    fn refresh_games(&mut self) {
        // Rescan the WBFS directory for games
        // Games are in the format "GAME TITLE [GAMEID]/"
        self.games = std::fs::read_dir(&self.wbfs_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    path.is_dir().then(|| Game::from_path(path))
                })
            })
            .collect();
    }

    pub fn add_isos(&mut self) {
        let iso_path = rfd::FileDialog::new()
            .set_title("Select ISO File")
            .add_filter("ISO Files", &["iso"])
            .pick_files();

        if let Some(paths) = iso_path {
            // TODO
        }
    }

    pub fn remove_game(&mut self, game: &Game) {
        // Remove the game from the WBFS directory
        let confirm = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                game.display_title
            ))
            .show();

        if confirm == rfd::MessageDialogResult::Yes {
            if std::fs::remove_dir_all(&game.path).is_err() {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description("Failed to remove the game.")
                    .show();
            }
        }

        self.refresh_games();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.button("➕ Add Game")
                    .on_hover_text("Add a new game to the WBFS directory");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} games", self.games.len()));
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // scrollable list of games
            egui::ScrollArea::vertical().show(ui, |ui| {
                for game in &self.games {
                    ui.horizontal(|ui| {
                        ui.label(game.display_title.clone());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.button("🗑️")
                                .on_hover_text("Remove this game from the WBFS directory");
                        });
                    });
                }
            });
        });
    }
}
