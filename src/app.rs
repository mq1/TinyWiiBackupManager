// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::base_dir::BaseDir;
use crate::convert::verify_game;
use crate::game::{CalculatedHashes, VerificationStatus};
use crate::messages::BackgroundMessage;
use crate::update_check::UpdateInfo;
use crate::{
    SUPPORTED_INPUT_EXTENSIONS, components::console_filter::ConsoleFilter, convert, game::Game,
    update_check,
};
use anyhow::Result;
use eframe::egui;
use egui_inbox::UiInbox;

/// Type of background operation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    Converting,
    Verifying,
}

/// Unified state for background operations (conversion and verification)
#[derive(Debug, Clone, PartialEq, Default)]
pub enum OperationState {
    /// No operation is in progress
    #[default]
    Idle,
    /// Operation is in progress
    InProgress {
        /// Type of operation
        operation: OperationType,
        /// Total number of items to process
        total_items: usize,
        /// Number of items already completed
        items_completed: usize,
        /// Current item being processed (for display)
        current_item: String,
        /// Current item progress (current / total)
        current_progress: (u64, u64),
        /// Items that passed (for verification only)
        items_passed: usize,
        /// Items that failed (for verification only)
        items_failed: usize,
    },
}

/// Result of a background operation
#[derive(Debug)]
pub enum OperationResult {
    ConversionComplete(CalculatedHashes),
    VerificationComplete(VerificationStatus),
    Error(anyhow::Error),
}

/// Main application state and UI controller.
#[derive(Default)]
pub struct App {
    /// Directory where the "wbfs" and "games" directories are located
    pub base_dir: Option<BaseDir>,
    /// List of discovered games
    pub games: Vec<Game>,
    /// WBFS dir size
    pub base_dir_size: u64,
    /// Inbox for receiving messages from background tasks
    pub inbox: UiInbox<BackgroundMessage>,
    /// Current state of background operations
    pub operation_state: OperationState,
    /// File watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Whether to remove sources after conversion
    pub remove_sources: bool,
    /// Set of game indices with open info windows
    pub open_info_windows: HashSet<usize>,
    /// Console filter state
    pub console_filter: ConsoleFilter,
    /// Update info
    pub update_info: Option<UpdateInfo>,
    /// Toasts
    pub top_left_toasts: egui_notify::Toasts,
    pub bottom_left_toasts: egui_notify::Toasts,
    pub bottom_right_toasts: egui_notify::Toasts,
    /// Cancellation flag for background operations
    pub operation_cancelled: Arc<AtomicBool>,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Initialize app and toasts
        let mut app = Self {
            top_left_toasts: create_toasts(egui_notify::Anchor::TopLeft),
            bottom_left_toasts: create_toasts(egui_notify::Anchor::BottomLeft),
            bottom_right_toasts: create_toasts(egui_notify::Anchor::BottomRight),
            operation_cancelled: Arc::new(AtomicBool::new(false)),
            ..Default::default()
        };

        // Load base dir from storage
        if let Some(storage) = cc.storage {
            app.base_dir = eframe::get_value(storage, "base_dir");
        }

        // If the base directory isn't set or no longer exists, prompt the user to select one.
        if app.base_dir.as_ref().is_none_or(|dir| !dir.exists()) {
            app.prompt_for_base_directory();
        }

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
            app.spawn_update_checker();
        };

        let sender = app.inbox.sender();
        if let Err(e) = app.watch_base_dir() {
            let _ = sender.send(BackgroundMessage::Error(e));
        }
        if let Err(e) = app.refresh_games() {
            let _ = sender.send(BackgroundMessage::Error(e));
        }

        app
    }

    fn prompt_for_base_directory(&mut self) {
        self.top_left_toasts
            .custom(
                "Click on \"📄 File\" to select a Drive/Directory    ",
                "⬆".to_string(),
                egui::Color32::DARK_GRAY,
            )
            .closable(false)
            .duration(None);
    }

    fn watch_base_dir(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            let sender = self.inbox.sender();
            let watcher = base_dir.get_watcher(move |res| {
                if let Ok(notify::Event {
                    kind:
                        notify::EventKind::Modify(_)
                        | notify::EventKind::Create(_)
                        | notify::EventKind::Remove(_),
                    ..
                }) = res
                {
                    let _ = sender.send(BackgroundMessage::DirectoryChanged);
                }
            })?;

            self.watcher = Some(watcher);
        }

        Ok(())
    }

    pub fn choose_base_dir(&mut self) -> Result<()> {
        let new_dir = rfd::FileDialog::new()
            .set_title("Select New Base Directory")
            .pick_folder();

        if let Some(new_dir) = new_dir {
            let base_dir = BaseDir::new(new_dir)?;
            self.base_dir = Some(base_dir);

            self.watch_base_dir()?;
            self.refresh_games()?;
        }

        Ok(())
    }

    pub fn refresh_games(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            // Save existing verification data
            let mut verification_cache = HashMap::new();
            for game in &self.games {
                if let Some(ref data) = game.verification_data {
                    verification_cache.insert(game.path.clone(), data.clone());
                }
            }

            // Load new games
            (self.games, self.base_dir_size) = base_dir.get_games()?;

            // Restore verification data and check if still valid
            for game in &mut self.games {
                if let Some(data) = verification_cache.get(&game.path)
                    && game.size == data.verified_size
                    && let Some(current_mtime) = game.get_latest_mtime()
                    && current_mtime == data.verified_mtime
                {
                    // Files haven't changed, restore verification
                    game.verification_data = Some(data.clone());
                }
            }
        }

        Ok(())
    }

    /// Opens a file dialog to select Wii Disc files and starts the conversion process.
    pub fn add_isos(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select Wii/GC Disc File(s)")
            .add_filter("Wii/GC Disc", SUPPORTED_INPUT_EXTENSIONS)
            .pick_files();

        if let Some(paths) = paths {
            self.spawn_conversion_worker(paths);
        }
    }

    pub fn spawn_update_checker(&mut self) {
        let sender = self.inbox.sender();
        std::thread::spawn(move || match update_check::check_for_new_version() {
            Ok(update_info) => sender.send(BackgroundMessage::UpdateCheckComplete(update_info)),
            Err(e) => sender.send(BackgroundMessage::Error(e)),
        });
    }

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        if let Some(base_dir) = &self.base_dir {
            let sender = self.inbox.sender();
            let remove_sources = self.remove_sources;
            let cancelled = Arc::clone(&self.operation_cancelled);

            // Reset cancellation flag
            self.operation_cancelled.store(false, Ordering::Relaxed);

            self.operation_state = OperationState::InProgress {
                operation: OperationType::Converting,
                total_items: paths.len(),
                items_completed: 0,
                current_item: String::new(),
                current_progress: (0, 0),
                items_passed: 0,
                items_failed: 0,
            };

            let base_dir = base_dir.path().to_owned();
            std::thread::spawn(move || {
                for (i, path) in paths.iter().enumerate() {
                    // Check for cancellation before starting each file
                    if cancelled.load(Ordering::Relaxed) {
                        break;
                    }

                    // Send the file name we're about to convert
                    if let Some(file_name) = path.file_name() {
                        let _ = sender.send(BackgroundMessage::OperationStartItem(
                            i,
                            file_name.to_string_lossy().to_string(),
                        ));
                    }

                    let mut success = false;
                    let (game_path, result) = match convert::convert_game(
                        path,
                        &base_dir,
                        sender.clone(),
                        Arc::clone(&cancelled),
                    ) {
                        Ok((game_dir, calculated_hashes)) => {
                            success = true;
                            (
                                game_dir,
                                OperationResult::ConversionComplete(calculated_hashes),
                            )
                        }
                        Err(e) => (base_dir.clone(), OperationResult::Error(e)),
                    };

                    // Send completion message for this file
                    let _ =
                        sender.send(BackgroundMessage::OperationItemComplete(game_path, result));

                    // Remove the source file if requested
                    if success
                        && remove_sources
                        && let Err(e) = fs::remove_file(path)
                    {
                        let _ = sender.send(BackgroundMessage::Error(e.into()));
                    }
                }

                let _ = sender.send(BackgroundMessage::OperationComplete);
            });
        }
    }

    /// Opens an info window for the specified game
    pub fn open_game_info(&mut self, index: usize) {
        // HashSet will automatically handle duplicates
        self.open_info_windows.insert(index);
    }

    /// Spawn a verification task for multiple games
    pub fn spawn_verification(&mut self, games: Vec<Box<Game>>) {
        let sender = self.inbox.sender();
        let cancelled = Arc::clone(&self.operation_cancelled);

        // Reset cancellation flag
        self.operation_cancelled.store(false, Ordering::Relaxed);

        // Set the operation state
        self.operation_state = OperationState::InProgress {
            operation: OperationType::Verifying,
            total_items: games.len(),
            items_completed: 0,
            current_item: String::new(),
            current_progress: (0, 0),
            items_passed: 0,
            items_failed: 0,
        };

        std::thread::spawn(move || {
            for (i, game) in games.into_iter().enumerate() {
                // Check for cancellation
                if cancelled.load(Ordering::Relaxed) {
                    break;
                }

                // Send the game name we're about to verify
                let _ = sender.send(BackgroundMessage::OperationStartItem(
                    i,
                    game.display_title.clone(),
                ));

                let game_path = game.path.clone();
                let result = match verify_game(game, sender.clone(), Arc::clone(&cancelled)) {
                    Ok(verification_status) => {
                        OperationResult::VerificationComplete(verification_status)
                    }
                    Err(e) => OperationResult::Error(e),
                };

                // Send completion message for this game
                let _ = sender.send(BackgroundMessage::OperationItemComplete(game_path, result));
            }

            // Send final completion message
            let _ = sender.send(BackgroundMessage::OperationComplete);
        });
    }

    /// Start verifying all unverified games
    pub fn start_verify_all(&mut self) {
        // Collect all games that need verification
        let mut games_to_verify: Vec<Box<Game>> = Vec::new();
        for game in &self.games {
            if !matches!(
                game.get_verification_status(),
                VerificationStatus::FullyVerified(_, _)
            ) {
                games_to_verify.push(Box::new(game.clone()));
            }
        }

        if games_to_verify.is_empty() {
            return;
        }

        // Start batch verification
        self.spawn_verification(games_to_verify);
    }
}

/// Helper function to create a styled `Toasts` instance for a specific screen corner.
fn create_toasts(anchor: egui_notify::Anchor) -> egui_notify::Toasts {
    egui_notify::Toasts::default()
        .with_anchor(anchor)
        .with_margin(egui::vec2(10.0, 32.0))
        .with_shadow(egui::Shadow {
            offset: [0, 0],
            blur: 0,
            spread: 1,
            color: egui::Color32::GRAY,
        })
}
