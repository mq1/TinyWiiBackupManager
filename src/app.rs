// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use iso2wbfs::ProgressUpdate;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::game::Game;
use egui::{ImageSource, RichText};
use poll_promise::Promise;

// --- UI Constants ---
const CARD_SIZE: egui::Vec2 = egui::vec2(180.0, 240.0);
const GRID_SPACING: egui::Vec2 = egui::vec2(10.0, 10.0);

/// Progress state for the conversion process.
#[derive(Clone)]
pub struct ConversionProgress {
    pub current_file: String,
    pub is_scrubbing: bool,
    pub total_blocks: u64,
    pub current_block: u64,
    pub error: Option<String>,
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            current_file: "Initializing conversion...".to_string(),
            is_scrubbing: false,
            total_blocks: 1,
            current_block: 0,
            error: None,
        }
    }
}

/// Result of the conversion process.
pub type ConversionResult = Result<(), String>;

pub struct App {
    wbfs_dir: PathBuf,
    games: Vec<Game>,
    conversion_promise: Option<Promise<ConversionResult>>,
    conversion_progress: Arc<Mutex<ConversionProgress>>,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            conversion_promise: None,
            conversion_progress: Arc::new(Mutex::new(ConversionProgress::default())),
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
                path.is_dir().then(|| Game::from_path(path))
            })
            .collect();
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
            self.refresh_games();
        }
    }

    // ========================
    // Conversion Process Logic
    // ========================

    /// Opens a file dialog to select ISOs and starts the conversion process.
    pub fn add_isos(&mut self) {
        if let Some(paths) = rfd::FileDialog::new()
            .set_title("Select ISO File(s)")
            .add_filter("ISO Files", &["iso"])
            .pick_files()
            .filter(|p| !p.is_empty())
        {
            self.spawn_conversion_worker(paths);
        }
    }

    /// Spawns a background thread to handle ISO to WBFS conversion.
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let wbfs_dir = self.wbfs_dir.clone();
        let progress = self.conversion_progress.clone();

        let promise = Promise::spawn_thread("conversion", move || {
            for path in paths {
                let file_path_str = path.display().to_string();

                let progress_callback = |update: ProgressUpdate| {
                    let mut progress = progress.lock().unwrap();
                    match update {
                        ProgressUpdate::ScrubbingStart => progress.is_scrubbing = true,
                        ProgressUpdate::ConversionStart { total_blocks } => {
                            progress.is_scrubbing = false;
                            progress.total_blocks = total_blocks;
                            progress.current_block = 0;
                        }
                        ProgressUpdate::ConversionUpdate { current_block } => {
                            progress.current_block = current_block;
                        }
                        ProgressUpdate::Done => {} // Single file done
                    }
                    progress.current_file = file_path_str.clone();
                };

                if let Err(e) = iso2wbfs::WbfsConverter::new(&path, &wbfs_dir)
                    .and_then(|mut converter| converter.convert(Some(progress_callback)))
                {
                    let mut progress = progress.lock().unwrap();
                    progress.error = Some(e.to_string());
                    return Err(e.to_string());
                }
            }
            Ok(())
        });

        self.conversion_promise = Some(promise);
    }

    /// Handles incoming messages from the conversion worker thread.
    fn handle_conversion_messages(&mut self, ctx: &egui::Context) {
        if let Some(promise) = &self.conversion_promise {
            if promise.ready().is_some() {
                let result = self.conversion_promise.take().unwrap().block_and_take();
                if let Err(e) = result {
                    rfd::MessageDialog::new()
                        .set_title("Conversion Error")
                        .set_description(&format!("Failed to convert: {}", e))
                        .show();
                }
                self.refresh_games();
            }
        }
        ctx.request_repaint();
    }

    // =============
    // UI Components
    // =============

    /// Renders the top menu bar
    fn ui_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_converting = self.conversion_promise.is_some();

                // Add games button
                ui.add_enabled_ui(!is_converting, |ui| {
                    if ui
                        .button("➕ Add Game(s)")
                        .on_hover_text("Add a new game to the WBFS directory")
                        .clicked()
                    {
                        self.add_isos();
                    }
                });

                // Game counter
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{} games", self.games.len()));
                });
            });
        });
    }

    /// Renders the main content area
    fn ui_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui_game_grid(ui);
        });
    }

    /// Renders the conversion progress modal using egui::Modal
    fn ui_conversion_modal(&self, ctx: &egui::Context) {
        if self.conversion_promise.is_none() {
            return;
        }

        // Create the modal dialog
        let modal = egui::Modal::new("conversion_modal".into());

        modal.show(ctx, |ui| {
            // Create a centered area for our content
            ui.vertical_centered(|ui| {
                // Title
                ui.heading("Converting ISOs");
                ui.separator();

                // Current file
                let progress = self.conversion_progress.lock().unwrap();
                ui.label(&progress.current_file);
                ui.add_space(10.0);

                // Progress indicator
                if progress.is_scrubbing {
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() / 3.0);
                        ui.spinner();
                        ui.label("Scrubbing disc...");
                    });
                } else {
                    let progress_value = if progress.total_blocks > 0 {
                        progress.current_block as f32 / progress.total_blocks as f32
                    } else {
                        0.0
                    };

                    ui.add(egui::ProgressBar::new(progress_value).show_percentage());
                    ui.label(format!(
                        "{} / {} blocks",
                        progress.current_block, progress.total_blocks
                    ));
                }
            });
        });
    }

    /// Renders the grid of available games
    fn ui_game_grid(&mut self, ui: &mut egui::Ui) {
        let mut game_index_to_remove = None;
        let num_columns = (ui.available_width() / CARD_SIZE.x).floor() as usize;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("games_grid")
                .num_columns(num_columns)
                .spacing(GRID_SPACING)
                .show(ui, |ui| {
                    // Store games in a temporary variable to avoid borrowing self
                    let games = &self.games;
                    for (i, game) in games.iter().enumerate() {
                        Self::ui_game_card(ui, game, || {
                            game_index_to_remove = Some(i);
                        });

                        if (i + 1) % num_columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        });

        if let Some(index) = game_index_to_remove {
            let game = self.games[index].clone();
            self.remove_game(&game);
        }
    }

    /// Renders a single game card (now a static method)
    fn ui_game_card(ui: &mut egui::Ui, game: &Game, on_remove: impl FnOnce()) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_max_size(CARD_SIZE);
            ui.vertical_centered(|ui| {
                // Game cover image
                let image_url = format!("https://art.gametdb.com/wii/cover/EN/{}.png", game.id);
                let image_source = ImageSource::Uri(Cow::Owned(image_url));
                let image = egui::Image::new(image_source)
                    .max_height(140.0)
                    .maintain_aspect_ratio(true)
                    .show_loading_spinner(true);
                ui.add(image);

                // Game info
                ui.add_space(5.0);
                ui.label(RichText::new(&game.display_title).strong());
                ui.label(
                    RichText::new(format!("ID: {}", game.id))
                        .monospace()
                        .size(12.0),
                );

                // Spacer to push button to bottom
                ui.add_space(ui.available_height() - 35.0);

                // Remove button
                if ui.button("🗑 Remove").clicked() {
                    on_remove();
                }
            });
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process background messages
        self.handle_conversion_messages(ctx);

        // Update cursor based on state
        ctx.set_cursor_icon(if self.conversion_promise.is_some() {
            egui::CursorIcon::Wait
        } else {
            egui::CursorIcon::Default
        });

        // Render UI components
        self.ui_top_panel(ctx);
        self.ui_central_panel(ctx);
        self.ui_conversion_modal(ctx);
    }
}
