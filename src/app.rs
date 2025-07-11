// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use iso2wbfs::ProgressUpdate;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::game::Game;
use egui::ImageSource;

/// Message from the conversion worker thread to the GUI thread.
pub enum ConversionMessage {
    /// Reports progress on a specific file.
    Update {
        file_path: String,
        update: ProgressUpdate,
    },
    /// Reports an error for a specific file.
    Error {
        file_path: String,
        error: String,
    },
    /// Signals that all files have been processed.
    Finished,
}

/// Holds the state of the background conversion process.
struct ConversionProcess {
    receiver: mpsc::Receiver<ConversionMessage>,
    current_file: String,
    is_scrubbing: bool,
    total_blocks: u64,
    current_block: u64,
}

pub struct App {
    wbfs_dir: PathBuf,
    games: Vec<Game>,
    conversion_process: Option<ConversionProcess>,
    // The manual image cache is no longer needed.
    // egui's internal image loader handles caching automatically.
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            conversion_process: None,
        };
        app.refresh_games();

        app
    }

    fn refresh_games(&mut self) {
        // Rescan the WBFS directory for games
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
        let iso_paths = rfd::FileDialog::new()
            .set_title("Select ISO File(s)")
            .add_filter("ISO Files", &["iso"])
            .pick_files();

        if let Some(paths) = iso_paths {
            if paths.is_empty() {
                return;
            }

            let (sender, receiver) = mpsc::channel();

            self.conversion_process = Some(ConversionProcess {
                receiver,
                current_file: "Initializing conversion...".to_string(),
                is_scrubbing: false,
                total_blocks: 1,
                current_block: 0,
            });

            let wbfs_dir_clone = self.wbfs_dir.clone();

            std::thread::spawn(move || {
                for path in paths {
                    let sender = sender.clone();
                    let file_path_str = path.display().to_string();

                    let progress_callback = |update: ProgressUpdate| {
                        // We don't care if the receiver has been dropped
                        let _ = sender.send(ConversionMessage::Update {
                            file_path: file_path_str.clone(),
                            update,
                        });
                    };

                    match iso2wbfs::WbfsConverter::new(&path, &wbfs_dir_clone) {
                        Ok(mut converter) => {
                            if let Err(e) = converter.convert(Some(progress_callback)) {
                                let _ = sender.send(ConversionMessage::Error {
                                    file_path: file_path_str,
                                    error: e.to_string(),
                                });
                            }
                        }
                        Err(e) => {
                            let _ = sender.send(ConversionMessage::Error {
                                file_path: file_path_str,
                                error: e.to_string(),
                            });
                        }
                    };
                }
                let _ = sender.send(ConversionMessage::Finished);
            });
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
            .set_buttons(rfd::MessageButtons::YesNo)
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
        // First, handle any background conversion messages
        if let Some(process) = &mut self.conversion_process {
            // Drain the channel of all pending messages
            while let Ok(msg) = process.receiver.try_recv() {
                match msg {
                    ConversionMessage::Update { file_path, update } => {
                        process.current_file = file_path;
                        match update {
                            ProgressUpdate::ScrubbingStart => {
                                process.is_scrubbing = true;
                            }
                            ProgressUpdate::ConversionStart { total_blocks } => {
                                process.is_scrubbing = false;
                                process.total_blocks = total_blocks;
                                process.current_block = 0;
                            }
                            ProgressUpdate::ConversionUpdate { current_block } => {
                                process.current_block = current_block;
                            }
                            ProgressUpdate::Done => {
                                // A single file is done, the worker will start the next
                            }
                        }
                    }
                    ConversionMessage::Error { file_path, error } => {
                        rfd::MessageDialog::new()
                            .set_title("Conversion Error")
                            .set_description(&format!(
                                "Failed to convert {}:\n{}",
                                file_path, error
                            ))
                            .show();
                    }
                    ConversionMessage::Finished => {
                        self.conversion_process = None;
                        self.refresh_games();
                        // Break the loop since self.conversion_process is now None
                        break;
                    }
                }
            }
            // After processing messages, request a repaint to show the changes.
            // This is crucial for the progress bar to update without user interaction.
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_converting = self.conversion_process.is_some();
                ui.add_enabled_ui(!is_converting, |ui| {
                    if ui
                        .button("➕ Add Game(s)")
                        .on_hover_text("Add a new game to the WBFS directory")
                        .clicked()
                    {
                        self.add_isos();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} games", self.games.len()));
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(process) = &self.conversion_process {
                // --- DRAW PROGRESS UI ---
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.3); // Push content down
                    ui.heading("Converting ISOs");
                    ui.separator();
                    ui.label(&process.current_file);
                    ui.add_space(10.0);

                    if process.is_scrubbing {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Scrubbing disc...");
                        });
                    } else {
                        let progress = if process.total_blocks > 0 {
                            process.current_block as f32 / process.total_blocks as f32
                        } else {
                            0.0
                        };
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                        ui.label(format!(
                            "{} / {} blocks",
                            process.current_block, process.total_blocks
                        ));
                    }
                });
            } else {
                // --- DRAW GAME GRID UI ---
                let mut game_to_remove = None;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Define the fixed size for each game card
                    let card_size = egui::vec2(180.0, 240.0);
                    // Calculate how many columns can fit in the available width
                    let num_columns = (ui.available_width() / card_size.x).floor() as usize;

                    // Create a grid with the calculated number of columns
                    egui::Grid::new("games_grid")
                        .num_columns(num_columns)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            for (i, game) in self.games.clone().iter().enumerate() {
                                // --- Create a single game card ---
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.set_max_size(card_size);
                                    ui.vertical_centered(|ui| {
                                        // --- Use egui::Image and ImageSource::Uri ---
                                        let image_url = format!(
                                            "https://art.gametdb.com/wii/cover/EN/{}.png",
                                            game.id
                                        );
                                        let image_source =
                                            ImageSource::Uri(Cow::Owned(image_url));

                                        let image = egui::Image::new(image_source)
                                            .max_height(140.0)
                                            .maintain_aspect_ratio(true)
                                            .show_loading_spinner(true);

                                        ui.add(image);

                                        // Game Title
                                        ui.add_space(5.0);
                                        ui.label(
                                            egui::RichText::new(game.display_title.clone())
                                                .strong(),
                                        );

                                        // Game ID
                                        ui.label(egui::RichText::new(format!(
                                            "ID: {}",
                                            game.id
                                        ))
                                            .monospace()
                                            .size(12.0));

                                        // Spacer to push button to the bottom
                                        ui.add_space(ui.available_height() - 35.0);

                                        // Remove Button
                                        if ui.button("🗑 Remove").clicked() {
                                            game_to_remove = Some(game.clone());
                                        }
                                    });
                                });

                                // End the row after the last column is filled
                                if (i + 1) % num_columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });

                if let Some(game) = game_to_remove {
                    self.remove_game(&game);
                }
            }
        });
    }
}