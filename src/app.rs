// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{error_handling, game::Game, version_check::{self, UpdateInfo}};
use anyhow::{Context as AnyhowContext, Result};
use eframe::egui::{self, CentralPanel};
use iso2wbfs::ProgressUpdate;
use poll_promise::Promise;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// Tracks the progress of the ISO conversion process.
#[derive(Default, Clone)]
pub struct ConversionProgress {
    pub current_file: String,
    pub is_scrubbing: bool,
    pub total_blocks: u64, // Default 0
    pub current_block: u64, // Default 0
}

/// Type alias for the conversion result.
pub type ConversionResult = Result<()>;

/// Main application state.
pub struct App {
    wbfs_dir: PathBuf,
    pub games: Vec<Arc<Game>>, // Use Arc for efficient removal/cloning
    pub conversion_promise: Option<Promise<ConversionResult>>,
    pub conversion_progress: Arc<Mutex<ConversionProgress>>,
    version_check_promise: Option<Promise<Result<Option<UpdateInfo>>>>,
    pub version_check_result: Option<UpdateInfo>,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            conversion_promise: None,
            conversion_progress: Arc::new(Mutex::new(ConversionProgress::default())),
            version_check_promise: Some(Promise::spawn_thread(
                "version_check",
                version_check::check_for_new_version,
            )),
            version_check_result: None,
        };
        app.refresh_games(); // Populate games on startup
        app
    }

    /// Scans the WBFS directory and updates the list of games.
    fn refresh_games(&mut self) {
        self.games = std::fs::read_dir(&self.wbfs_dir)
            .into_iter() // Convert Result to iterator
            .flatten()
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    // Check if it's a directory and try to parse it as a Game
                    path.is_dir().then(|| Game::from_path(path).ok().map(Arc::new)).flatten()
                })
            })
            .collect();
    }

    /// Prompts the user and removes a game from the filesystem.
    pub fn remove_game(&mut self, game_to_remove: &Arc<Game>) {
        // Use a more concise dialog creation and check result in one line
        if rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                game_to_remove.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            == rfd::MessageDialogResult::Yes
        {
            // Attempt to remove the directory
            if let Err(e) = std::fs::remove_dir_all(&game_to_remove.path) {
                error_handling::show_error("Error", &format!("Failed to remove game: {}", e));
            } else {
                // Only refresh if removal was successful
                self.refresh_games();
            }
        }
    }

    /// Opens a file dialog to select ISOs and starts the conversion process.
    pub fn add_isos(&mut self) {
        // Use filter to check for non-empty selection
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

        // Spawn the conversion thread and store the promise
        self.conversion_promise = Some(Promise::spawn_thread("conversion", move || {
            for path in paths {
                let file_path_str = path.display().to_string();
                // Create a callback to update progress
                let progress_callback = |update: ProgressUpdate| {
                    // Safely acquire the lock and update progress
                    if let Ok(mut progress) = progress.lock() { // Use if let for lock result
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
                        // Update the current file being processed
                        progress.current_file = file_path_str.clone();
                    }
                    // If lock fails, we silently ignore progress updates (not ideal, but prevents panic)
                };

                // Perform the conversion with error context
                iso2wbfs::WbfsConverter::new(&path, &wbfs_dir)
                    .and_then(|mut converter| converter.convert(Some(progress_callback)))
                    .with_context(|| format!("Failed to convert {}", path.display()))?;
            }
            Ok(()) // Return Ok if all conversions succeeded
        }));
    }

    /// Handles incoming messages from the conversion worker thread.
    fn handle_conversion_messages(&mut self, ctx: &egui::Context) {
        // Take ownership of the promise to check its state
        if let Some(promise) = self.conversion_promise.take() {
            // Use try_take to check if the promise is ready without blocking
            match promise.try_take() {
                Ok(result) => {
                    // Conversion finished
                    if let Err(e) = result {
                        // Show error if conversion failed
                        error_handling::show_error(
                            "Conversion Error",
                            &format!("Failed to convert: {:#}", e),
                        );
                    }
                    // Refresh the game list regardless of success/failure
                    self.refresh_games();
                }
                Err(promise) => {
                    // Conversion still in progress, put the promise back
                    self.conversion_promise = Some(promise);
                }
            }
        }
        // Always request a repaint if we might be waiting for a promise
        ctx.request_repaint();
    }

    /// Handles the result of the version check.
    fn handle_version_check(&mut self) {
        // Take ownership of the promise to check its state
        if let Some(promise) = self.version_check_promise.take() {
            // Use try_take to check if the promise is ready without blocking
            match promise.try_take() {
                Ok(Ok(Some(update_info))) => {
                    // New version found
                    self.version_check_result = Some(update_info);
                }
                Ok(Err(e)) => {
                    // Error during version check
                    error_handling::show_error(
                        "Version Check Error",
                        &format!("Failed to check for new version: {:#}", e),
                    );
                }
                // Ok(None) means no update, Err(promise) means still pending.
                // In both cases, we don't need to do anything further.
                _ => {}
            }
            // If the promise was ready (taken), we leave version_check_promise as None.
            // If it was still pending (Err(promise)), we would put it back.
            // However, version check is a one-time thing, so we can leave it as None.
            // If we wanted to retry, we'd need to put the Err(promise) back.
            // For a one-time check, this is simpler.
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process background messages from worker threads
        self.handle_conversion_messages(ctx);
        self.handle_version_check();

        // Update the mouse cursor based on application state
        ctx.set_cursor_icon(if self.conversion_promise.is_some() {
            egui::CursorIcon::Wait
        } else {
            egui::CursorIcon::Default
        });

        // Render UI components
        crate::components::top_panel::ui_top_panel(ctx, self);
        CentralPanel::default().show(ctx, |ui| {
            crate::components::game_grid::ui_game_grid(ui, self);
        });
        crate::components::conversion_modal::ui_conversion_modal(ctx, self);
        crate::components::update_notification_panel::ui_update_notification_panel(ctx, self);
    }
}