// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use eframe::egui;
use egui_inbox::UiInbox;

use crate::base_dir::BaseDir;
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
            (self.games, self.base_dir_size) = base_dir.get_games()?;
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

            self.conversion_state = ConversionState::Converting {
                total_files: paths.len(),
                files_converted: 0,
                current_progress: (0, 0),
            };

            let base_dir = base_dir.path().to_owned();
            std::thread::spawn(move || {
                for path in paths {
                    let progress_callback = |progress, total| {
                        let _ = sender.send(BackgroundMessage::ConversionProgress(progress, total));
                    };

                    if let Err(e) = iso2wbfs::convert(&path, &base_dir, progress_callback) {
                        let _ = sender.send(BackgroundMessage::Error(e.into()));
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
