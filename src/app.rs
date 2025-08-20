// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use eframe::egui;
use egui_inbox::UiInbox;
use notify::{RecursiveMode, Watcher};

use crate::{
    components,
    error_handling::show_anyhow_error,
    game::Game,
    version_check::{self, UpdateInfo},
};

// don't format
#[rustfmt::skip]
const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &[
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
    /// Signal that a single file conversion has completed
    FileConverted,
    /// Signal that the conversion has completed (with result)
    ConversionComplete(Result<()>),
    /// Signal that the version check has completed
    VersionCheckComplete(Result<Option<UpdateInfo>>),
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
    /// Result of the version check, if available
    pub version_check_result: Option<UpdateInfo>,
    /// File watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Whether to remove sources after conversion
    pub remove_sources: bool,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(_cc: &eframe::CreationContext<'_>, base_dir: PathBuf) -> Result<Self> {
        let mut app = Self {
            base_dir,
            ..Default::default()
        };

        app.spawn_dir_watcher()?;
        app.spawn_version_check();
        app.refresh_games()?;
        Ok(app)
    }

    /// Spawns a file watcher for the base directory
    fn spawn_dir_watcher(&mut self) -> Result<()> {
        let sender = self.inbox.sender();

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(_) = res {
                let _ = sender.send(BackgroundMessage::DirectoryChanged);
            }
        })?;

        watcher.watch(&self.base_dir, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }

    /// Spawns a background thread to check for application updates.
    fn spawn_version_check(&self) {
        let sender = self.inbox.sender();

        std::thread::spawn(move || {
            let result = version_check::check_for_new_version();
            let _ = sender.send(BackgroundMessage::VersionCheckComplete(result));
        });
    }

    fn scan_dir(&self, dir_name: &str) -> Result<Vec<Game>> {
        let dir = self.base_dir.join(dir_name);

        let mut games = Vec::new();
        if !dir.is_dir() {
            return Ok(games);
        }

        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                games.push(Game::from_path(path)?);
            }
        }

        Ok(games)
    }

    /// Scans the "wbfs" and "games" directories and updates the list of games.
    pub fn refresh_games(&mut self) -> Result<()> {
        let wii_games = self.scan_dir("wbfs")?;
        let gc_games = self.scan_dir("games")?;

        self.games = [wii_games, gc_games].concat();
        self.games
            .sort_by(|a, b| a.display_title.cmp(&b.display_title));

        // get base_dir_size using fs_extra
        self.base_dir_size = fs_extra::dir::get_size(&self.base_dir)
            .with_context(|| format!("Failed to get size of dir: {}", self.base_dir.display()))?;

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
        };

        std::thread::spawn(move || {
            for path in paths {
                if let Err(e) = Self::convert_single_iso(&path, &base_dir) {
                    let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e)));
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

    /// Converts a single ISO file to WBFS format
    fn convert_single_iso(path: &PathBuf, base_dir: &PathBuf) -> Result<()> {
        iso2wbfs::convert(path, base_dir).map_err(Error::from)
    }

    /// Processes messages received from background tasks
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.read(ctx) {
            match msg {
                BackgroundMessage::FileConverted => {
                    if let ConversionState::Converting {
                        total_files,
                        files_converted,
                    } = self.conversion_state
                    {
                        self.conversion_state = ConversionState::Converting {
                            total_files,
                            files_converted: files_converted + 1,
                        };
                    }
                }

                BackgroundMessage::ConversionComplete(result) => {
                    self.conversion_state = ConversionState::Idle;
                    if let Err(e) = result {
                        show_anyhow_error("Conversion Failed", &e);
                    }
                }

                BackgroundMessage::VersionCheckComplete(result) => {
                    // Silently ignore errors
                    if let Ok(update) = result {
                        self.version_check_result = update;
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
    }
}

fn set_cursor_icon(ctx: &egui::Context, app: &App) {
    match app.conversion_state {
        ConversionState::Converting { .. } => ctx.set_cursor_icon(egui::CursorIcon::Wait),
        _ => ctx.set_cursor_icon(egui::CursorIcon::Default),
    }
}
