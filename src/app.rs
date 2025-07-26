// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use anyhow::{Context, Result};
use eframe::egui;
use egui_inbox::UiInbox;

use crate::{
    error_handling,
    game::Game,
    version_check::{self, UpdateInfo},
};

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
enum BackgroundMessage {
    /// Signal that a single file conversion has completed
    FileConverted,
    /// Signal that the conversion has completed (with result)
    ConversionComplete(Result<()>),
    /// Signal that the version check has completed
    VersionCheckComplete(Result<Option<UpdateInfo>>),
}

/// Main application state and UI controller.
pub struct App {
    /// Directory where WBFS files are stored
    wbfs_dir: PathBuf,
    /// List of discovered games
    pub games: Vec<Game>,
    /// Inbox for receiving messages from background tasks
    inbox: UiInbox<BackgroundMessage>,
    /// Whether a conversion is currently in progress
    pub conversion_in_progress: bool,
    /// Total number of files to convert
    pub total_files_to_convert: usize,
    /// Number of files already converted
    pub files_converted: usize,
    /// Result of the version check, if available
    pub version_check_result: Option<UpdateInfo>,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let inbox = UiInbox::new();

        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            inbox,
            conversion_in_progress: false,
            total_files_to_convert: 0,
            files_converted: 0,
            version_check_result: None,
        };

        app.spawn_version_check();
        app.refresh_games();
        app
    }

    /// Spawns a background thread to check for application updates.
    fn spawn_version_check(&mut self) {
        let sender = self.inbox.sender();

        std::thread::spawn(move || {
            let result = version_check::check_for_new_version();
            let _ = sender.send(BackgroundMessage::VersionCheckComplete(result));
        });
    }

    /// Scans the WBFS directory and updates the list of games.
    fn refresh_games(&mut self) {
        let entries = match std::fs::read_dir(&self.wbfs_dir) {
            Ok(entries) => entries,
            Err(e) => {
                error_handling::show_error("Error", &format!("Failed to read WBFS directory: {e}"));
                std::process::exit(1);
            }
        };

        self.games = entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_dir() {
                    Game::from_path(path).ok()
                } else {
                    None
                }
            })
            .collect();
    }

    /// Prompts the user to confirm game removal and removes it if confirmed.
    pub fn remove_game(&mut self, game_to_remove: &Game) {
        let confirmed = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!("Are you sure you want to remove {}?", game_to_remove.display_title))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show() == rfd::MessageDialogResult::Yes;

        if !confirmed {
            return;
        }

        if let Err(e) = std::fs::remove_dir_all(&game_to_remove.path) {
            error_handling::show_error("Error", &format!("Failed to remove game: {e}"));
        } else {
            self.refresh_games();
        }
    }

    /// Opens a file dialog to select ISO files and starts the conversion process.
    pub fn add_isos(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select ISO File(s)")
            .add_filter("ISO Files", &["iso"])
            .pick_files();

        if let Some(paths) = paths.filter(|p| !p.is_empty()) {
            self.spawn_conversion_worker(paths);
        }
    }

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let wbfs_dir = self.wbfs_dir.clone();
        let sender = self.inbox.sender();

        self.conversion_in_progress = true;
        self.total_files_to_convert = paths.len();
        self.files_converted = 0;

        std::thread::spawn(move || {
            for path in paths {
                if let Err(e) = Self::convert_single_iso(&path, &wbfs_dir) {
                    let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e)));
                    return;
                }
                let _ = sender.send(BackgroundMessage::FileConverted);
            }

            let _ = sender.send(BackgroundMessage::ConversionComplete(Ok(())));
        });
    }

    /// Converts a single ISO file to WBFS format
    fn convert_single_iso(path: &PathBuf, wbfs_dir: &PathBuf) -> Result<()> {
        let mut converter = iso2wbfs::WbfsConverter::new(path, wbfs_dir)
            .with_context(|| format!("Failed to initialize converter for {}", path.display()))?;

        converter.convert()?;

        Ok(())
    }

    /// Processes messages received from background tasks
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.read(ctx) {
            match msg {
                BackgroundMessage::FileConverted => {
                    self.files_converted += 1;
                }

                BackgroundMessage::ConversionComplete(result) => {
                    self.conversion_in_progress = false;
                    match result {
                        Ok(()) => self.refresh_games(),
                        Err(e) => error_handling::show_error("Conversion Failed", &e.to_string()),
                    }
                }

                BackgroundMessage::VersionCheckComplete(result) => {
                    self.handle_version_check_result(result);
                }
            }
        }
    }

    /// Handles the result of a version check
    fn handle_version_check_result(&mut self, result: Result<Option<UpdateInfo>>) {
        match result {
            Ok(Some(update)) => self.version_check_result = Some(update),
            Ok(None) => self.version_check_result = None,
            Err(e) => {
                self.version_check_result = None;
                error_handling::show_error("Update Check Failed", &e.to_string());
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use crate::components::{
            conversion_modal, game_grid, top_panel, update_notification_panel,
        };

        self.handle_messages(ctx);
        self.update_cursor_icon(ctx);

        top_panel::ui_top_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            game_grid::ui_game_grid(ui, self);
            conversion_modal::ui_conversion_modal(ctx, self);
            update_notification_panel::ui_update_notification_panel(ctx, self);
        });
    }
}

impl App {
    /// Updates the cursor icon based on the application state
    fn update_cursor_icon(&self, ctx: &egui::Context) {
        let cursor_icon = if self.conversion_in_progress {
            egui::CursorIcon::Wait
        } else {
            egui::CursorIcon::Default
        };
        ctx.set_cursor_icon(cursor_icon);
    }
}
