// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use eframe::egui;
use egui_inbox::UiInbox;

use crate::{
    components,
    error_handling::show_anyhow_error,
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
    pub wbfs_dir: PathBuf,
    /// List of discovered games
    pub games: Vec<Game>,
    /// WBFS dir size
    pub wbfs_dir_size: u64,
    /// Inbox for receiving messages from background tasks
    inbox: UiInbox<BackgroundMessage>,
    /// Whether a conversion is currently in progress
    pub is_converting: bool,
    /// Total number of files to convert
    pub total_files_to_convert: usize,
    /// Number of files already converted
    pub files_converted: usize,
    /// Result of the version check, if available
    pub version_check_result: Option<UpdateInfo>,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(_cc: &eframe::CreationContext<'_>, wbfs_dir: PathBuf) -> Result<Self> {
        let inbox = UiInbox::new();

        let mut app = Self {
            wbfs_dir,
            games: Vec::new(),
            wbfs_dir_size: 0,
            inbox,
            is_converting: false,
            total_files_to_convert: 0,
            files_converted: 0,
            version_check_result: None,
        };

        app.spawn_version_check();
        app.refresh_games()?;
        Ok(app)
    }

    /// Spawns a background thread to check for application updates.
    fn spawn_version_check(&self) {
        let sender = self.inbox.sender();

        std::thread::spawn(move || {
            let result = version_check::check_for_new_version();
            let _ = sender.send(BackgroundMessage::VersionCheckComplete(result));
        });
    }

    /// Scans the WBFS directory and updates the list of games.
    fn refresh_games(&mut self) -> Result<()> {
        let entries = std::fs::read_dir(&self.wbfs_dir)
            .with_context(|| format!("Failed to read dir: {}", self.wbfs_dir.display()))?;

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

        // get wbfs_dir_size using fs_extra
        self.wbfs_dir_size = fs_extra::dir::get_size(&self.wbfs_dir)
            .with_context(|| format!("Failed to get size of dir: {}", self.wbfs_dir.display()))?;

        Ok(())
    }

    /// Prompts the user to confirm game removal and removes it if confirmed.
    pub fn remove_game(&mut self, game_to_remove: &Game) {
        let res = rfd::MessageDialog::new()
            .set_title("Remove Game")
            .set_description(format!(
                "Are you sure you want to remove {}?",
                game_to_remove.display_title
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if res == rfd::MessageDialogResult::No {
            return;
        }

        if let Err(e) = self.try_remove_game(game_to_remove) {
            show_anyhow_error("Error", &e);
        } else {
            if let Err(e) = self.refresh_games() {
                show_anyhow_error("Error", &e);
            }
        }
    }

    fn try_remove_game(&mut self, game_to_remove: &Game) -> Result<()> {
        std::fs::remove_dir_all(&game_to_remove.path)
            .with_context(|| format!("Failed to remove game: {}", game_to_remove.path.display()))
    }

    /// Opens a file dialog to select Wii Disc files and starts the conversion process.
    pub fn add_isos(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select Wii Disc File(s)")
            .add_filter(
                "Wii Disc",
                &[
                    "iso", "ISO", "wbfs", "WBFS", "wia", "WIA", "rvz", "RVZ", "ciso", "CISO",
                    "gcz", "GCZ",
                ],
            )
            .pick_files();

        if let Some(paths) = paths.filter(|p| !p.is_empty()) {
            self.spawn_conversion_worker(paths);
        }
    }

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let wbfs_dir = self.wbfs_dir.clone();
        let sender = self.inbox.sender();

        self.is_converting = true;
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
        iso2wbfs::convert(path, wbfs_dir).map_err(Error::from)
    }

    /// Processes messages received from background tasks
    fn handle_messages(&mut self, ctx: &egui::Context) {
        for msg in self.inbox.read(ctx) {
            match msg {
                BackgroundMessage::FileConverted => {
                    self.files_converted += 1;
                }

                BackgroundMessage::ConversionComplete(result) => {
                    self.is_converting = false;
                    match result {
                        Ok(()) => {
                            if let Err(e) = self.refresh_games() {
                                show_anyhow_error("Error", &e);
                            }
                        }
                        Err(e) => show_anyhow_error("Conversion Failed", &e),
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
                show_anyhow_error("Update Check Failed", &e);
            }
        }
    }

    #[cfg(target_os = "macos")]
    /// Run dot_clean in the parent directory
    pub fn run_dot_clean(&self) -> Result<()> {
        let parent_dir = self
            .wbfs_dir
            .parent()
            .context("Failed to get parent directory")?;

        let confirm = rfd::MessageDialog::new()
            .set_title("Run dot_clean")
            .set_description(format!(
                "Are you sure you want to run dot_clean in {}?",
                parent_dir.display()
            ))
            .set_buttons(rfd::MessageButtons::OkCancel)
            .show();

        if confirm == rfd::MessageDialogResult::Ok {
            std::process::Command::new("dot_clean")
                .arg("-m")
                .arg(parent_dir)
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

            if self.is_converting {
                components::conversion_modal::ui_conversion_modal(ctx, self);
            }
        });
    }
}

fn set_cursor_icon(ctx: &egui::Context, app: &App) {
    if app.is_converting {
        ctx.set_cursor_icon(egui::CursorIcon::Wait);
    } else {
        ctx.set_cursor_icon(egui::CursorIcon::Default);
    }
}
