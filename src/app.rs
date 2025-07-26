// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{
    error_handling,
    game::Game,
    version_check::{self, UpdateInfo},
};
use anyhow::{Context as AnyhowContext, Result};
use eframe::egui;
use egui_inbox::UiInbox;
use iso2wbfs::ProgressUpdate;
use std::path::PathBuf;

/// Tracks the progress of the ISO conversion process.
#[derive(Debug, Default, Clone)]
pub struct ConversionProgress {
    pub current_file: String,
    pub is_scrubbing: bool,
    pub total_blocks: u64,  // Default 0
    pub current_block: u64, // Default 0
}

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
enum BackgroundMessage {
    /// Update the progress of the current conversion
    ConversionProgress(ConversionProgress),
    /// Signal that the conversion has completed (with result)
    ConversionComplete(Result<()>),
    /// Signal that the version check has completed
    VersionCheckComplete(Result<Option<UpdateInfo>>),
}

/// Main application state.
pub struct App {
    wbfs_dir: PathBuf,
    pub games: Vec<Game>,
    /// Inbox for receiving messages from background tasks
    inbox: UiInbox<BackgroundMessage>,
    /// Whether a conversion is currently in progress
    pub conversion_in_progress: bool,
    /// Progress of the current conversion
    pub conversion_progress: ConversionProgress,
    /// Result of the version check, if available
    pub version_check_result: Option<UpdateInfo>,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    #[must_use]
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Self {
        let inbox = UiInbox::new();

        // Spawn version check in the background
        let version_check_sender = inbox.sender();
        std::thread::spawn(move || {
            let result = version_check::check_for_new_version();
            let _ = version_check_sender.send(BackgroundMessage::VersionCheckComplete(result));
        });

        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            inbox,
            conversion_in_progress: false,
            conversion_progress: ConversionProgress::default(),
            version_check_result: None,
        };

        app.refresh_games(); // Populate games on startup
        app
    }

    /// Scans the WBFS directory and updates the list of games.
    fn refresh_games(&mut self) {
        self.games = std::fs::read_dir(&self.wbfs_dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    path.is_dir().then(|| Game::from_path(path).ok()).flatten()
                })
            })
            .collect();
    }

    /// Prompts the user and removes a game from the filesystem.
    pub fn remove_game(&mut self, game_to_remove: &Game) {
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
                error_handling::show_error("Error", &format!("Failed to remove game: {e}"));
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

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let wbfs_dir = self.wbfs_dir.clone();
        let sender = self.inbox.sender();

        self.conversion_in_progress = true;
        self.conversion_progress = ConversionProgress::default();

        std::thread::spawn(move || {
            let result = Ok(());

            for path in paths {
                let file_path = path.display().to_string();

                match (|| -> Result<()> {
                    let progress = ConversionProgress {
                        current_file: file_path,
                        ..Default::default()
                    };
                    let mut converter = iso2wbfs::WbfsConverter::new(&path, &wbfs_dir)?;

                    let sender_clone = sender.clone();
                    converter.convert(Some(move |update| {
                        let mut progress = progress.clone();
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
                            ProgressUpdate::Done => {}
                        }
                        let _ = sender_clone.send(BackgroundMessage::ConversionProgress(progress));
                    }))?;

                    Ok(())
                })()
                .with_context(|| format!("Failed to convert {}", path.display()))
                {
                    Ok(()) => {}
                    Err(e) => {
                        let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e)));
                        return;
                    }
                }
            }

            if let Err(e) = result {
                let _ = sender.send(BackgroundMessage::ConversionComplete(Err(e)));
            } else {
                let _ = sender.send(BackgroundMessage::ConversionComplete(Ok(())));
            }
        });
    }

    // Process messages from background tasks
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.read(ctx) {
            match msg {
                BackgroundMessage::ConversionProgress(progress) => {
                    self.conversion_progress = progress;
                }

                BackgroundMessage::ConversionComplete(result) => {
                    self.conversion_in_progress = false;
                    match result {
                        Ok(_) => self.refresh_games(),
                        Err(e) => error_handling::show_error("Conversion Failed", &e.to_string()),
                    }
                }

                BackgroundMessage::VersionCheckComplete(result) => {
                    match &result {
                        Ok(Some(update)) => self.version_check_result = Some(update.clone()),
                        Ok(None) => self.version_check_result = None,
                        Err(e) => {
                            self.version_check_result = None;
                            error_handling::show_error("Update Check Failed", &e.to_string());
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use crate::components::*;

        self.handle_messages(ctx);

        let cursor_icon = if self.conversion_in_progress {
            egui::CursorIcon::Wait
        } else {
            egui::CursorIcon::Default
        };
        ctx.set_cursor_icon(cursor_icon);

        egui::CentralPanel::default().show(ctx, |ui| {
            top_panel::ui_top_panel(ctx, self);
            game_grid::ui_game_grid(ui, self);
            conversion_modal::ui_conversion_modal(ctx, self);
            update_notification_panel::ui_update_notification_panel(ctx, self);
        });
    }
}
