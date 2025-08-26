// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use anyhow::Result;
use eframe::egui;
use egui_inbox::UiInbox;

use crate::base_dir::BaseDir;
use crate::convert::verify_game;
use crate::game::VerificationStatus;
use crate::messages::BackgroundMessage;
use crate::update_check::UpdateInfo;
use crate::{
    SUPPORTED_INPUT_EXTENSIONS, components::console_filter::ConsoleFilter, game::Game, update_check,
};

/// State of the conversion process
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ConversionState {
    /// No conversion is in progress
    #[default]
    Idle,
    /// Conversion is in progress
    Converting {
        /// Total number of files to convert
        total_files: usize,
        /// Number of files already converted
        files_converted: usize,
        /// Current file progress (current / total)
        current_progress: (u64, u64),
    },
}

/// State of the verification process
#[derive(Debug, Clone, PartialEq, Default)]
pub enum VerificationState {
    /// No verification is in progress
    #[default]
    Idle,
    /// Verification is in progress
    Verifying {
        /// Path of the game being verified
        game_path: PathBuf,
        /// Current progress (current / total)
        progress: (u64, u64),
        /// Queue of remaining games to verify
        queue: Vec<PathBuf>,
        /// Total games to verify (for overall progress)
        total_games: usize,
        /// Games already processed
        games_verified: usize,
        /// Games that passed verification
        games_passed: usize,
        /// Games that failed verification
        games_failed: usize,
    },
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
    /// Current state of the conversion process
    pub conversion_state: ConversionState,
    /// Current state of the verification process
    pub verification_state: VerificationState,
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

        // Pretty notifications
        let bottom_right_toasts = egui_notify::Toasts::default()
            .with_anchor(egui_notify::Anchor::BottomRight)
            .with_margin(egui::Vec2::new(10.0, 32.))
            .with_shadow(egui::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 1,
                color: egui::Color32::GRAY,
            });

        let mut top_left_toasts = egui_notify::Toasts::default()
            .with_anchor(egui_notify::Anchor::TopLeft)
            .with_margin(egui::Vec2::new(10.0, 32.0))
            .with_shadow(egui::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 1,
                color: egui::Color32::GRAY,
            });

        let bottom_left_toasts = egui_notify::Toasts::default()
            .with_anchor(egui_notify::Anchor::BottomLeft)
            .with_margin(egui::Vec2::new(10.0, 32.0))
            .with_shadow(egui::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 1,
                color: egui::Color32::GRAY,
            });

        // Show toast to choose base dir
        top_left_toasts
            .custom(
                "Click on \"📄 File\" to select a Drive/Directory    ",
                "⬆".to_string(),
                egui::Color32::DARK_GRAY,
            )
            .closable(false)
            .duration(None);

        let mut app = Self {
            top_left_toasts,
            bottom_left_toasts,
            bottom_right_toasts,
            operation_cancelled: Arc::new(AtomicBool::new(false)),
            ..Default::default()
        };

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
            app.spawn_update_checker();
        };

        app
    }

    pub fn choose_base_dir(&mut self) -> Result<()> {
        let new_dir = rfd::FileDialog::new()
            .set_title("Select New Base Directory")
            .pick_folder();

        if let Some(new_dir) = new_dir {
            let base_dir = BaseDir::new(new_dir)?;

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
            self.base_dir = Some(base_dir);

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

            self.conversion_state = ConversionState::Converting {
                total_files: paths.len(),
                files_converted: 0,
                current_progress: (0, 0),
            };

            let base_dir = base_dir.path().to_owned();
            std::thread::spawn(move || {
                for path in paths {
                    // Check for cancellation before starting each file
                    if cancelled.load(Ordering::Relaxed) {
                        let _ = sender.send(BackgroundMessage::ConversionComplete);
                        return;
                    }

                    let progress_callback = |progress, total| {
                        let _ = sender.send(BackgroundMessage::ConversionProgress(progress, total));
                    };

                    if let Err(e) = iso2wbfs::convert(&path, &base_dir, progress_callback) {
                        let _ = sender.send(BackgroundMessage::Error(e.into()));
                    }

                    // Check if conversion was cancelled after completing this file
                    if cancelled.load(Ordering::Relaxed) {
                        let _ = sender.send(BackgroundMessage::ConversionComplete);
                        return;
                    }

                    let _ = sender.send(BackgroundMessage::FileConverted);

                    // remove the source file
                    if remove_sources && let Err(e) = std::fs::remove_file(&path) {
                        let _ = sender.send(BackgroundMessage::Error(e.into()));
                    }
                }

                let _ = sender.send(BackgroundMessage::ConversionComplete);
            });
        }
    }

    /// Opens an info window for the specified game
    pub fn open_game_info(&mut self, index: usize) {
        // HashSet will automatically handle duplicates
        self.open_info_windows.insert(index);
    }

    /// Spawn a verification task for a game
    pub fn spawn_verification(&mut self, game: Box<Game>) {
        let game_path = game.path.clone();
        let sender = self.inbox.sender();
        let cancelled = Arc::clone(&self.operation_cancelled);

        // Reset cancellation flag
        self.operation_cancelled.store(false, Ordering::Relaxed);

        std::thread::spawn(move || {
            let result = verify_game(game, sender.clone(), cancelled);
            let _ = sender.send(BackgroundMessage::VerificationComplete(game_path, result));
        });
    }

    /// Start verifying all unverified games
    pub fn start_verify_all(&mut self) {
        // Collect all games that need verification
        let mut queue: Vec<PathBuf> = Vec::new();
        for game in &self.games {
            if !matches!(
                game.get_verification_status(),
                VerificationStatus::FullyVerified(_, _)
            ) {
                queue.push(game.path.clone());
            }
        }

        if queue.is_empty() {
            return;
        }

        let total_games = queue.len();

        // Start verification of the first game
        if let Some(first_path) = queue.first()
            && let Some(game) = self
                .games
                .iter()
                .find(|g| &g.path == first_path)
                .map(|g| Box::new(g.clone()))
        {
            let mut remaining_queue = queue.clone();
            remaining_queue.remove(0);

            self.verification_state = VerificationState::Verifying {
                game_path: first_path.clone(),
                progress: (0, 0),
                queue: remaining_queue,
                total_games,
                games_verified: 0,
                games_passed: 0,
                games_failed: 0,
            };

            self.spawn_verification(game);
        }
    }
}
