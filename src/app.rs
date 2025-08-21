// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use eframe::egui;
use egui_inbox::UiInbox;
use egui_suspense::EguiSuspense;
use notify::{RecursiveMode, Watcher};

use crate::{
    components::{self, console_filter::ConsoleFilter, update_notifier::UpdateInfo},
    error_handling::show_anyhow_error,
    game::Game,
};

// don't format
#[rustfmt::skip]
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
    "gcm", "GCM",
    "iso", "ISO",
    "wbfs", "WBFS",
    "wia", "WIA",
    "rvz", "RVZ",
    "ciso", "CISO",
    "gcz", "GCZ",
    "tgc", "TGC",
    "nfs", "NFS",
];

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
enum BackgroundMessage {
    /// Signal for current file conversion progress
    ConversionProgress(u64, u64),
    /// Signal that a single file conversion has completed
    FileConverted,
    /// Signal that the conversion has completed (with result)
    ConversionComplete(Result<()>),
    /// Signal that the directory has changed
    DirectoryChanged,
}

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
    pub base_dir: PathBuf,
    /// List of discovered games
    pub games: Vec<Game>,
    /// WBFS dir size
    pub base_dir_size: u64,
    /// Inbox for receiving messages from background tasks
    inbox: UiInbox<BackgroundMessage>,
    /// Current state of the conversion process
    pub conversion_state: ConversionState,
    /// File watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Whether to remove sources after conversion
    pub remove_sources: bool,
    /// Update checker component
    pub update_checker: Option<EguiSuspense<Option<UpdateInfo>, Error>>,
    /// Vector of game indices with open info windows
    pub open_info_windows: Vec<usize>,
    /// Console filter state
    pub console_filter: ConsoleFilter,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        base_dir: PathBuf,
        updates_enabled: bool,
    ) -> Result<Self> {
        // Initialize the update checker based on the updates_enabled flag
        let update_checker = updates_enabled
            .then(|| EguiSuspense::single_try(components::update_notifier::check_for_new_version));

        let mut app = Self {
            base_dir,
            update_checker,
            ..Default::default()
        };

        app.spawn_dir_watcher()?;
        app.refresh_games().context("Failed to refresh games")?;
        Ok(app)
    }

    /// Spawns a file watcher for the base directory
    fn spawn_dir_watcher(&mut self) -> Result<()> {
        let sender = self.inbox.sender();

        let mut watcher = notify::recommended_watcher(move |res| {
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

        watcher.watch(&self.base_dir, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }

    fn scan_dir(&self, dir_name: &str, is_gc: bool) -> Result<Vec<Game>> {
        let dir = self.base_dir.join(dir_name);

        let mut games = Vec::new();
        if !dir.is_dir() {
            return Ok(games);
        }

        for entry in std::fs::read_dir(&dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(game) = Game::from_path(path, is_gc) {
                        games.push(game);
                    }
                }
            }
        }

        Ok(games)
    }

    /// Scans the "wbfs" and "games" directories and updates the list of games.
    pub fn refresh_games(&mut self) -> Result<()> {
        let wii_games = self.scan_dir("wbfs", false)?;
        let gc_games = self.scan_dir("games", true)?;

        // Combine for the main games vector
        self.games = [wii_games, gc_games].concat();
        
        // Sort the combined vector
        self.games
            .sort_by(|a, b| a.display_title.cmp(&b.display_title));

        // sum the sizes of each game object
        self.base_dir_size = self.games.iter().fold(0, |acc, game| acc + game.size);

        Ok(())
    }

    /// Opens a file dialog to select Wii Disc files and starts the conversion process.
    pub fn add_isos(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select Wii/GC Disc File(s)")
            .add_filter("Wii/GC Disc", SUPPORTED_INPUT_EXTENSIONS)
            .pick_files();

        if let Some(paths) = paths.filter(|p| !p.is_empty()) {
            self.spawn_conversion_worker(paths);
        }
    }

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let base_dir = self.base_dir.clone();
        let sender = self.inbox.sender();
        let remove_sources = self.remove_sources;

        self.conversion_state = ConversionState::Converting {
            total_files: paths.len(),
            files_converted: 0,
            current_progress: (0, 0),
        };

        std::thread::spawn(move || {
            for path in paths {
                if let Err(e) = iso2wbfs::convert(&path, &base_dir, |progress, total| {
                    let _ = sender.send(BackgroundMessage::ConversionProgress(progress, total));
                }) {
                    let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e.into())));
                    return;
                }
                let _ = sender.send(BackgroundMessage::FileConverted);

                // remove the source file
                if remove_sources {
                    if let Err(e) = std::fs::remove_file(&path) {
                        let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e.into())));
                        return;
                    }
                }
            }

            let _ = sender.send(BackgroundMessage::ConversionComplete(Ok(())));
        });
    }

    /// Processes messages received from background tasks
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.read(ctx) {
            match msg {
                BackgroundMessage::ConversionProgress(progress, total) => {
                    if let ConversionState::Converting {
                        total_files,
                        files_converted,
                        ..
                    } = self.conversion_state
                    {
                        self.conversion_state = ConversionState::Converting {
                            total_files,
                            files_converted,
                            current_progress: (progress, total),
                        };
                    }
                }

                BackgroundMessage::FileConverted => {
                    if let ConversionState::Converting {
                        total_files,
                        files_converted,
                        ..
                    } = self.conversion_state
                    {
                        self.conversion_state = ConversionState::Converting {
                            total_files,
                            files_converted: files_converted + 1,
                            current_progress: (0, 0),
                        };
                    }
                }

                BackgroundMessage::ConversionComplete(result) => {
                    self.conversion_state = ConversionState::Idle;
                    if let Err(e) = result {
                        show_anyhow_error("Conversion Failed", &e);
                    }
                }

                BackgroundMessage::DirectoryChanged => {
                    if let Err(e) = self.refresh_games() {
                        show_anyhow_error("Error", &e);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    /// Run dot_clean to clean up MacOS ._ files
    pub fn run_dot_clean(&self) -> Result<()> {
        let confirm = rfd::MessageDialog::new()
            .set_title("Run dot_clean")
            .set_description(format!(
                "Are you sure you want to run dot_clean in {}?",
                self.base_dir.display()
            ))
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show();

        if confirm == rfd::MessageDialogResult::Ok {
            std::process::Command::new("dot_clean")
                .arg("-m")
                .arg(&self.base_dir)
                .spawn()
                .context("Failed to run dot_clean")?;
        }

        Ok(())
    }

    /// Opens an info window for the specified game
    pub fn open_game_info(&mut self, index: usize) {
        // Only add the index if it's not already in the vector
        if !self.open_info_windows.contains(&index) {
            self.open_info_windows.push(index);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages(ctx);
        set_cursor_icon(ctx, self);

        components::top_panel::ui_top_panel(ctx, self);
        components::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            components::game_grid::ui_game_grid(ui, self);

            if matches!(self.conversion_state, ConversionState::Converting { .. }) {
                components::conversion_modal::ui_conversion_modal(ctx, self);
            }
        });

        // Render info windows for opened games
        self.open_info_windows.retain_mut(|&mut index| {
            self.games.get_mut(index).map_or(false, |game| {
                let mut is_open = true;
                components::game_info::ui_game_info_window(ctx, game, &mut is_open);
                is_open
            })
        });
    }
}

fn set_cursor_icon(ctx: &egui::Context, app: &App) {
    match app.conversion_state {
        ConversionState::Converting { .. } => ctx.set_cursor_icon(egui::CursorIcon::Wait),
        _ => ctx.set_cursor_icon(egui::CursorIcon::Default),
    }
}
