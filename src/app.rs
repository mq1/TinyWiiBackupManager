// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use iso2wbfs::ProgressUpdate;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::game::Game;
use egui::{ImageSource, RichText};

// --- Constants for UI Layout ---
const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 240.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

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
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            conversion_process: None,
        };
        app.refresh_games();
        app
    }

    /// Scans the WBFS directory and updates the list of games.
    fn refresh_games(&mut self) {
        self.games = std::fs::read_dir(&self.wbfs_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_dir() {
                    Some(Game::from_path(path))
                } else {
                    None
                }
            })
            .collect();
    }

    /// Opens a file dialog to select ISOs and starts the conversion process.
    pub fn add_isos(&mut self) {
        let iso_paths = rfd::FileDialog::new()
            .set_title("Select ISO File(s)")
            .add_filter("ISO Files", &["iso"])
            .pick_files();

        if let Some(paths) = iso_paths {
            if !paths.is_empty() {
                self.spawn_conversion_worker(paths);
            }
        }
    }

    /// Spawns a background thread to handle ISO to WBFS conversion.
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let (sender, receiver) = mpsc::channel();
        self.conversion_process = Some(ConversionProcess {
            receiver,
            current_file: "Initializing conversion...".to_string(),
            is_scrubbing: false,
            total_blocks: 1,
            current_block: 0,
        });

        let wbfs_dir = self.wbfs_dir.clone();

        std::thread::spawn(move || {
            for path in paths {
                let file_path_str = path.display().to_string();
                let sender = sender.clone();

                let progress_callback = |update: ProgressUpdate| {
                    // We don't care if the receiver has been dropped
                    let _ = sender.send(ConversionMessage::Update {
                        file_path: file_path_str.clone(),
                        update,
                    });
                };

                // Perform conversion for a single file and handle potential errors
                let result = iso2wbfs::WbfsConverter::new(&path, &wbfs_dir)
                    .and_then(|mut converter| converter.convert(Some(progress_callback)));

                if let Err(e) = result {
                    let _ = sender.send(ConversionMessage::Error {
                        file_path: file_path_str,
                        error: e.to_string(),
                    });
                }
            }
            // Signal that all conversions are finished
            let _ = sender.send(ConversionMessage::Finished);
        });
    }

    /// Prompts the user and removes a game from the filesystem.
    pub fn remove_game(&mut self, game_to_remove: &Game) {
        let confirm = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                game_to_remove.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if confirm == rfd::MessageDialogResult::Yes {
            if let Err(e) = std::fs::remove_dir_all(&game_to_remove.path) {
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(format!("Failed to remove game: {}", e))
                    .show();
            }
        }

        self.refresh_games();
    }

    /// Handles incoming messages from the conversion worker thread.
    fn handle_conversion_messages(&mut self, ctx: &egui::Context) {
        if let Some(process) = &mut self.conversion_process {
            // Drain the channel of all pending messages for this frame
            while let Ok(msg) = process.receiver.try_recv() {
                match msg {
                    ConversionMessage::Update { file_path, update } => {
                        process.current_file = file_path;
                        match update {
                            ProgressUpdate::ScrubbingStart => process.is_scrubbing = true,
                            ProgressUpdate::ConversionStart { total_blocks } => {
                                process.is_scrubbing = false;
                                process.total_blocks = total_blocks;
                                process.current_block = 0;
                            }
                            ProgressUpdate::ConversionUpdate { current_block } => {
                                process.current_block = current_block;
                            }
                            ProgressUpdate::Done => {
                                // A single file is done, worker will start the next
                            }
                        }
                    }
                    ConversionMessage::Error { file_path, error } => {
                        rfd::MessageDialog::new()
                            .set_title("Conversion Error")
                            .set_description(&format!("Failed to convert {}:\n{}", file_path, error))
                            .show();
                    }
                    ConversionMessage::Finished => {
                        self.conversion_process = None;
                        self.refresh_games();
                        break; // Stop processing messages as the process is now gone
                    }
                }
            }
            // Request a repaint to ensure the progress bar updates smoothly
            ctx.request_repaint();
        }
    }

    /// Renders the top menu bar.
    fn ui_top_panel(&mut self, ctx: &egui::Context) {
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
    }

    /// Renders the main content area.
    fn ui_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_game_grid(ui);
        });
    }

    /// Renders the conversion progress modal.
    fn ui_conversion_modal(&self, ctx: &egui::Context) {
        if let Some(process) = &self.conversion_process {
            // Create semi-transparent overlay to block background interaction
            egui::Area::new("conversion_modal_background".into())
                .order(egui::Order::Foreground)
                .interactable(true)
                .show(ctx, |ui| {
                    let rect = ui.max_rect();
                    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_black_alpha(128));
                });

            // Create centered modal window
            let screen_rect = ctx.available_rect();
            let center = screen_rect.center();
            let window_size = egui::vec2(400.0, 150.0);
            let window_rect = egui::Rect::from_center_size(center, window_size);
            
            egui::Area::new("conversion_modal_window".into())
                .order(egui::Order::Tooltip) // Highest layer
                .fixed_pos(window_rect.min)
                .show(ctx, |ui| {
                    egui::Frame::window(ui.style()).show(ui, |ui| {
                        ui.set_width(window_size.x);
                        ui.set_height(window_size.y);
                        ui.vertical_centered(|ui| {
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
                    });
                });
        }
    }

    /// Renders the grid of available games.
    fn ui_game_grid(&mut self, ui: &mut egui::Ui) {
        let mut game_index_to_remove = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            let num_columns = (ui.available_width() / CARD_SIZE.x).floor() as usize;

            egui::Grid::new("games_grid")
                .num_columns(num_columns)
                .spacing(GRID_SPACING)
                .show(ui, |ui| {
                    for (i, game) in self.games.iter().enumerate() {
                        ui_game_card(ui, game, || {
                            game_index_to_remove = Some(i);
                        });

                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        if let Some(index) = game_index_to_remove {
            // Clone only the game we need to remove, outside the main render loop.
            let game = self.games[index].clone();
            self.remove_game(&game);
        }
    }
}

/// Renders a single game card as a stateless UI function.
fn ui_game_card(ui: &mut egui::Ui, game: &Game, on_remove: impl FnOnce()) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_max_size(CARD_SIZE);
        ui.vertical_centered(|ui| {
            let image_url = format!("https://art.gametdb.com/wii/cover/EN/{}.png", game.id);
            let image_source = ImageSource::Uri(Cow::Owned(image_url));
            let image = egui::Image::new(image_source)
                .max_height(140.0)
                .maintain_aspect_ratio(true)
                .show_loading_spinner(true);
            ui.add(image);

            ui.add_space(5.0);
            ui.label(RichText::new(&game.display_title).strong());
            ui.label(RichText::new(format!("ID: {}", game.id)).monospace().size(12.0));

            // Spacer to push the button to the bottom
            ui.add_space(ui.available_height() - 35.0);

            if ui.button("🗑 Remove").clicked() {
                on_remove();
            }
        });
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Handle background messages first
        self.handle_conversion_messages(ctx);

        // Set cursor based on conversion state
        if self.conversion_process.is_some() {
            ctx.set_cursor_icon(egui::CursorIcon::Wait);
        } else {
            ctx.set_cursor_icon(egui::CursorIcon::Default);
        }

        // 2. Draw the UI panels
        self.ui_top_panel(ctx);
        self.ui_central_panel(ctx);
        
        // 3. Show conversion modal if needed
        self.ui_conversion_modal(ctx);
    }
}