// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::{Context as AnyhowContext, Result};
use iso2wbfs::ProgressUpdate;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::game::Game;
use crate::version_check::{self, UpdateInfo};
use egui::CentralPanel;
use poll_promise::Promise;

/// Progress state for the conversion process.
#[derive(Clone)]
pub struct ConversionProgress {
    pub current_file: String,
    pub is_scrubbing: bool,
    pub total_blocks: u64,
    pub current_block: u64,
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            current_file: "Initializing conversion...".to_string(),
            is_scrubbing: false,
            total_blocks: 1,
            current_block: 0,
        }
    }
}

/// Result of the conversion process.
pub type ConversionResult = Result<()>;

pub struct App {
    wbfs_dir: PathBuf,
    pub games: Vec<Game>,
    pub conversion_promise: Option<Promise<ConversionResult>>,
    pub conversion_progress: Arc<Mutex<ConversionProgress>>,
    version_check_promise: Option<Promise<Result<Option<UpdateInfo>>>>,
    pub version_check_result: Option<UpdateInfo>,
}

impl App {
    /// Creates a new instance of the application.
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let version_check_promise = Some(Promise::spawn_thread("version_check", || {
            version_check::check_for_new_version()
        }));

        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            conversion_promise: None,
            conversion_progress: Arc::new(Mutex::new(ConversionProgress::default())),
            version_check_promise,
            version_check_result: None,
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

        let promise = Promise::spawn_thread("conversion", move || -> Result<()> {
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

                iso2wbfs::WbfsConverter::new(&path, &wbfs_dir)
                    .and_then(|mut converter| converter.convert(Some(progress_callback)))
                    .with_context(|| format!("Failed to convert {}", path.display()))?;
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

    // ======================
    // Version Check Logic
    // ======================

    fn handle_version_check(&mut self) {
        if let Some(promise) = &self.version_check_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(Some(update_info)) => {
                        self.version_check_result = Some(update_info.clone());
                    }
                    Ok(None) => {
                        log::info!("You are running the latest version.");
                    }
                    Err(e) => {
                        log::error!("Failed to check for new version: {}", e);
                    }
                }
                self.version_check_promise = None;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process background messages
        self.handle_conversion_messages(ctx);
        self.handle_version_check();

        // Update cursor based on state
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
