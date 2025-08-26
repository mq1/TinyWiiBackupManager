// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use eframe::egui;
use egui_inbox::UiInbox;

use crate::base_dir::BaseDir;
use crate::messages::BackgroundMessage;
use crate::settings::Settings;
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
    /// Application settings
    pub settings: Settings,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Load saved settings
        let settings = if let Some(storage) = cc.storage {
            Settings::load(storage)
        } else {
            Settings::default()
        };

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

        let top_left_toasts = egui_notify::Toasts::default()
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

        let mut app = Self {
            top_left_toasts,
            bottom_left_toasts,
            bottom_right_toasts,
            remove_sources: settings.remove_sources,
            settings,
            ..Default::default()
        };

        // Try to restore base directory from settings
        if let Some(base_dir_path) = app.settings.base_dir_path.clone()
            && base_dir_path.exists()
            && let Err(e) = app.set_base_dir(base_dir_path)
        {
            app.bottom_right_toasts
                .error(format!("Failed to load saved directory: {}", e))
                .closable(true)
                .duration(None);
        }

        // Only show directory selection prompt if no directory is loaded
        if app.base_dir.is_none() {
            app.top_left_toasts
                .custom(
                    "Click on \"📄 File\" to select a Drive/Directory    ",
                    "⬆".to_string(),
                    egui::Color32::DARK_GRAY,
                )
                .closable(false)
                .duration(None);
        }

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
            app.spawn_update_checker();
        };

        app
    }

    /// Set up a base directory with watcher and refresh games
    fn set_base_dir(&mut self, path: PathBuf) -> Result<()> {
        let base_dir = BaseDir::new(path.clone())?;

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

        // Save the base directory to settings
        self.settings.base_dir_path = Some(path);

        self.refresh_games()?;

        Ok(())
    }

    pub fn choose_base_dir(&mut self) -> Result<()> {
        let new_dir = rfd::FileDialog::new()
            .set_title("Select New Base Directory")
            .pick_folder();

        if let Some(new_dir) = new_dir {
            self.set_base_dir(new_dir)?;
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
