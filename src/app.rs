// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::base_dir::BaseDir;
use crate::cover_manager::CoverManager;
use crate::game::{Game, VerificationStatus};
use crate::jobs::check_update::{CheckUpdateConfig, CheckUpdateResult, start_check_update};
use crate::jobs::convert::{ConvertConfig, ConvertResult, start_convert};
use crate::jobs::download_covers::DownloadCoversResult;
use crate::jobs::download_database::DownloadDatabaseResult;
use crate::jobs::egui_waker::egui_waker;
use crate::jobs::verify::{VerifyConfig, VerifyResult, start_verify};
use crate::jobs::{Job, JobQueue, JobStatus};
use crate::messages::BackgroundMessage;
use crate::toasts::{create_toasts, error_toast};
use crate::update_check::UpdateInfo;
use crate::util::gametdb;
use crate::{SUPPORTED_INPUT_EXTENSIONS, components::console_filter::ConsoleFilter};
use anyhow::Result;
use eframe::egui;
use egui_inbox::UiInbox;

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
    /// Cover manager for downloading game covers
    pub cover_manager: Option<CoverManager>,
    /// Job queue for background tasks
    pub jobs: JobQueue,
    /// Whether the jobs window is open
    pub show_jobs_window: bool,
    /// Egui context for waking the UI
    pub ctx: egui::Context,
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
            cover_manager: None,
            ctx: cc.egui_ctx.clone(),
            ..Default::default()
        };

        // Load base dir from storage
        if let Some(storage) = cc.storage {
            app.base_dir = eframe::get_value(storage, "base_dir");

            // Initialize cover manager if we have a base directory
            if let Some(ref base_dir) = app.base_dir {
                app.cover_manager = Some(CoverManager::new(base_dir.path().to_path_buf()));
            }
        }

        // If the base directory isn't set or no longer exists, prompt the user to select one.
        if app.base_dir.as_ref().is_none_or(|dir| !dir.exists()) {
            app.prompt_for_base_directory();
        }

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
            app.spawn_update_checker();
        }

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
            let base_dir = BaseDir::new(new_dir.clone())?;
            self.base_dir = Some(base_dir);

            // Initialize cover manager with the new base directory
            self.cover_manager = Some(CoverManager::new(new_dir));

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
        self.jobs.push_once(Job::CheckUpdate, || {
            start_check_update(egui_waker(&self.ctx), CheckUpdateConfig {})
        });
    }

    /// Converts ISO files to WBFS
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        if let Some(base_dir) = &self.base_dir {
            self.jobs.push_once(Job::Convert, || {
                start_convert(
                    egui_waker(&self.ctx),
                    ConvertConfig {
                        base_dir: base_dir.path().to_owned(),
                        paths,
                        remove_sources: self.remove_sources,
                    },
                )
            });
        }
    }

    /// Opens an info window for the specified game
    pub fn open_game_info(&mut self, index: usize) {
        // HashSet will automatically handle duplicates
        self.open_info_windows.insert(index);
    }

    /// Spawn a verification task for multiple games
    pub fn spawn_verification(&mut self, games: Vec<Game>) {
        self.jobs.push_once(Job::Verify, || {
            start_verify(egui_waker(&self.ctx), VerifyConfig { games })
        });
    }

    /// Start verifying all unverified games
    pub fn start_verify_all(&mut self) {
        // Collect all games that need verification
        let mut games_to_verify: Vec<Game> = Vec::new();
        for game in &self.games {
            if !matches!(
                game.get_verification_status(),
                VerificationStatus::FullyVerified(_, _)
            ) {
                games_to_verify.push(game.clone());
            }
        }

        if games_to_verify.is_empty() {
            self.bottom_right_toasts
                .info("All games are already verified");
            return;
        }

        // Start batch verification
        self.spawn_verification(games_to_verify);
    }

    /// Handle download covers job result
    pub fn handle_download_covers_result(&mut self, status: JobStatus, res: DownloadCoversResult) {
        if res.failed > 0 {
            self.bottom_right_toasts.warning(status.status);
            // Mark failed IDs to avoid retrying 404s
            if let Some(cover_manager) = &self.cover_manager {
                cover_manager.mark_failed(res.failed_ids.clone(), res.cover_type);
            }
        } else {
            self.bottom_right_toasts.success(status.status);
        }
    }

    /// Handle convert job result
    pub fn handle_convert_result(&mut self, status: JobStatus, res: ConvertResult) {
        if res.failed.is_empty() {
            self.bottom_right_toasts.success(status.status);
        } else {
            for (failed_path, e) in &res.failed {
                self.bottom_right_toasts.add(error_toast(
                    &format!("Failed to convert {}", failed_path.display()),
                    e,
                ));
            }
            self.bottom_right_toasts.warning(status.status);
        }

        // Refresh the game list to show new converted games
        if let Err(e) = self.refresh_games() {
            self.bottom_right_toasts
                .add(error_toast("Failed to refresh games", &e));
        }

        // Apply calculated hashes to the newly converted games
        for (game_path, calculated_hashes) in res.hashes {
            if let Some(game) = self.games.iter_mut().find(|g| g.path == game_path) {
                game.set_verification_status(calculated_hashes.into_verification_status());
                break;
            }
        }
    }

    /// Handle verify job result
    pub fn handle_verify_result(&mut self, status: JobStatus, res: VerifyResult) {
        // Apply verification statuses to the games
        for (game_path, verification_status) in res.results {
            if let Some(game) = self.games.iter_mut().find(|g| g.path == game_path) {
                game.set_verification_status(verification_status);
            }
        }

        if res.failed > 0 {
            self.bottom_right_toasts.warning(status.status);
        } else {
            self.bottom_right_toasts.success(status.status);
        }
    }

    /// Handle download database job result
    pub fn handle_download_database_result(
        &mut self,
        status: JobStatus,
        _res: DownloadDatabaseResult,
    ) {
        // Clear the cached GameTDB instance to force reload
        gametdb::clear_cache();

        // Refresh games to reload titles from the new database
        if let Err(e) = self.refresh_games() {
            self.bottom_right_toasts
                .add(error_toast("Failed to refresh games", &e));
        } else {
            self.bottom_right_toasts.success(status.status);
        }
    }

    /// Handle check update job result
    pub fn handle_check_update_result(&mut self, _status: JobStatus, res: CheckUpdateResult) {
        if res.update_available
            && let (Some(version), Some(url)) = (res.version, res.url)
        {
            let update_text = format!("✨Update available: {}✨    ", version);

            self.bottom_left_toasts
                .custom(update_text, "⬇".to_string(), egui::Color32::GRAY);

            self.update_info = Some(UpdateInfo { version, url });
        }
    }
}
