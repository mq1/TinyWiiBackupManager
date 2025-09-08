// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::base_dir::BaseDir;
use crate::game::Game;
use crate::gui::toasts;
use crate::messages::BackgroundMessage;
use crate::settings::Settings;
use crate::task::TaskProcessor;
use crate::util::ext::SUPPORTED_INPUT_EXTENSIONS;
use crate::util::oscwii;
use crate::util::update_check::{UpdateInfo, check_for_new_version};
use crate::util::wiiapps::WiiApp;
use crate::{gui::console_filter::ConsoleFilter, util};
use anyhow::{Context, Result};
use eframe::egui;
use egui_inbox::UiInbox;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Games,
    WiiApps,
}

/// Main application state and UI controller.
pub struct App {
    /// Current view
    pub view: View,
    /// Directory where the "wbfs" and "games" directories are located
    pub base_dir: Option<BaseDir>,
    /// List of discovered games
    pub games: Vec<Game>,
    /// fs
    pub used_space: u64,
    pub total_space: u64,
    /// Inbox for receiving messages from background tasks
    pub inbox: UiInbox<BackgroundMessage>,
    /// File watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Whether to remove sources after conversion
    pub remove_sources: bool,
    /// Whether to remove sources after conversion
    pub remove_sources_wiiapps: bool,
    /// Console filter state
    pub console_filter: ConsoleFilter,
    /// Update info
    pub update_info: Option<UpdateInfo>,
    /// Toasts
    pub top_right_toasts: egui_notify::Toasts,
    pub bottom_left_toasts: egui_notify::Toasts,
    pub bottom_right_toasts: egui_notify::Toasts,
    /// Status
    pub task_status: Option<String>,
    /// Task processor
    pub task_processor: TaskProcessor,
    /// Settings
    pub settings: Settings,
    pub settings_window_open: bool,
    /// List of discovered apps
    pub wiiapps: Vec<WiiApp>,
    pub wiiload_window_open: bool,
    /// oscwii contents cache
    pub oscwii_apps: oscwii::AppCache,
    pub oscwii_window_open: bool,
    pub oscwii_filter: String,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Initialize inbox
        let inbox = UiInbox::new();
        let ui_sender = inbox.sender();

        // Initialize task processor
        let task_processor = TaskProcessor::new(ui_sender);

        // Load base dir from storage
        let base_dir = cc.storage.and_then(|storage| {
            let dir = eframe::get_value::<BaseDir>(storage, "base_dir");
            let version = eframe::get_value::<String>(storage, "app_version");

            // On macOS, only keep dir if version matches (to avoid popups)
            if cfg!(target_os = "macos") {
                match version {
                    Some(v) if v == APP_VERSION => dir,
                    _ => None,
                }
            } else {
                dir
            }
        });

        // Load settings from storage
        let settings = cc
            .storage
            .and_then(|storage| eframe::get_value::<Settings>(storage, "settings"))
            .unwrap_or_default();

        // Load oscwii cache from storage
        let oscwii_apps = cc
            .storage
            .and_then(|storage| eframe::get_value::<oscwii::AppCache>(storage, "oscwii_contents"))
            .unwrap_or_default();

        // Initialize app and toasts
        let mut app = Self {
            view: View::Games,
            top_right_toasts: toasts::create_toasts(egui_notify::Anchor::TopRight)
                .with_margin(egui::vec2(49.0, 36.0)),
            bottom_left_toasts: toasts::create_toasts(egui_notify::Anchor::BottomLeft),
            bottom_right_toasts: toasts::create_toasts(egui_notify::Anchor::BottomRight),
            inbox,
            task_processor,
            base_dir,
            games: Vec::new(),
            used_space: 0,
            total_space: 0,
            remove_sources: false,
            remove_sources_wiiapps: false,
            console_filter: ConsoleFilter::default(),
            update_info: None,
            watcher: None,
            task_status: None,
            settings,
            settings_window_open: false,
            wiiapps: Vec::new(),
            wiiload_window_open: false,
            oscwii_apps,
            oscwii_window_open: false,
            oscwii_filter: String::new(),
        };

        // If the base directory isn't set or no longer exists, prompt the user to select one.
        if app.base_dir.as_ref().is_none_or(|dir| !dir.exists()) {
            toasts::prompt_for_base_directory(&mut app);
        }

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        if std::env::var_os("TWBM_DISABLE_UPDATES").is_none() {
            app.spawn_check_for_new_version_task();
        }

        // Initialize the OSCWii app list
        if app.oscwii_apps.needs_update() {
            app.spawn_update_apps_task();
        }

        let sender = app.inbox.sender();
        if let Err(e) = app.watch_base_dir() {
            let _ = sender.send(e.into());
        }
        if let Err(e) = app.refresh_games() {
            let _ = sender.send(e.into());
        }
        if let Err(e) = app.refresh_wiiapps() {
            let _ = sender.send(e.into());
        }
        if let Err(e) = app.refresh_fs() {
            let _ = sender.send(e.into());
        }

        app
    }

    fn watch_base_dir(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            let sender = self.inbox.sender();
            let callback = move || {
                let _ = sender.send(BackgroundMessage::DirectoryChanged);
            };

            let watcher = base_dir.get_watcher(callback)?;
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
            self.refresh_wiiapps()?;
            self.refresh_fs()?;
        }

        Ok(())
    }

    pub fn refresh_fs(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            let free_space = fs2::free_space(base_dir.path())?;
            let total_space = fs2::total_space(base_dir.path())?;
            self.used_space = total_space - free_space;
            self.total_space = total_space;
        }

        Ok(())
    }

    pub fn refresh_games(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            self.games = base_dir.get_games()?;
            util::checksum::sync_games(&self.games);
        }

        Ok(())
    }

    pub fn refresh_wiiapps(&mut self) -> Result<()> {
        if let Some(base_dir) = &self.base_dir {
            self.wiiapps = util::wiiapps::get_installed(base_dir, &self.oscwii_apps)?;
        }

        Ok(())
    }

    pub fn download_covers(&mut self) {
        if let Some(base_dir) = &self.base_dir {
            for game in self.games.clone() {
                let base_dir = base_dir.clone();

                self.task_processor.spawn_task(move |ui_sender| {
                    if game.download_cover(&base_dir)? {
                        let _ = ui_sender.send(BackgroundMessage::NewCover(game));
                    }
                    Ok(())
                });
            }
        }
    }

    pub fn download_all_covers(&mut self) {
        if let Some(base_dir) = &self.base_dir {
            for game in self.games.iter() {
                let game = game.clone();
                let base_dir = base_dir.clone();

                self.task_processor.spawn_task(move |ui_sender| {
                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(format!(
                        "Downloading covers for {}",
                        game.display_title
                    )));

                    if game.download_all_covers(base_dir)? {
                        let msg = format!("Downloaded covers for {}", game.display_title);
                        let _ = ui_sender.send(BackgroundMessage::Info(msg));
                    }
                    Ok(())
                });
            }
        }
    }

    /// Opens a file dialog to select Wii Disc files and starts the conversion process.
    pub fn add_isos(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select Wii/GC Disc File(s)")
            .add_filter("Wii/GC Disc", SUPPORTED_INPUT_EXTENSIONS)
            .pick_files();

        if let Some(paths) = paths
            && let Some(base_dir) = &self.base_dir
        {
            let remove_sources = self.remove_sources;

            for path in paths {
                let base_dir = base_dir.clone();
                let wii_output_format = self.settings.wii_output_format;
                let strip_partitions = self.settings.strip_partitions;

                self.task_processor.spawn_task(move |ui_sender| {
                    let cloned_ui_sender = ui_sender.clone();
                    let file_name = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("");

                    util::convert::convert(
                        &path,
                        base_dir.path(),
                        wii_output_format,
                        strip_partitions,
                        move |current, total| {
                            let status = format!(
                                "📄➡🖴  {:02.0}%  {}",
                                current as f32 / total as f32 * 100.0,
                                file_name
                            );

                            let _ = cloned_ui_sender.send(BackgroundMessage::UpdateStatus(status));
                        },
                    )?;

                    let _ = ui_sender.send(BackgroundMessage::DirectoryChanged);
                    let _ =
                        ui_sender.send(BackgroundMessage::Info(format!("Converted {file_name}")));

                    if remove_sources {
                        std::fs::remove_file(&path)?;
                    }

                    Ok(())
                });
            }

            // Trigger the cover download
            self.task_processor.spawn_task(move |ui_sender| {
                let _ = ui_sender.send(BackgroundMessage::TriggerDownloadCovers);
                Ok(())
            })
        }
    }

    pub fn add_wiiapps(&mut self) {
        let paths = rfd::FileDialog::new()
            .set_title("Select Wii App(s)")
            .add_filter("Wii App", &["zip", "ZIP"])
            .pick_files();

        if let Some(paths) = paths
            && let Some(base_dir) = &self.base_dir
        {
            let remove_sources = self.remove_sources_wiiapps;

            for path in paths {
                let base_dir = base_dir.clone();

                self.task_processor.spawn_task(move |ui_sender| {
                    let _ = ui_sender.send(BackgroundMessage::UpdateStatus(format!(
                        "Adding Wii App: {}",
                        path.file_name().unwrap().to_str().unwrap()
                    )));

                    base_dir.add_zip(&path)?;

                    if remove_sources {
                        std::fs::remove_file(&path)?;
                    }

                    Ok(())
                });
            }
        }
    }

    pub fn total_integrity_check(&mut self) {
        for game in &self.games {
            game.spawn_integrity_check_task(&self.task_processor);
        }
    }

    pub fn spawn_download_database_task(&self) {
        if let Some(base_dir) = &self.base_dir {
            let base_dir = base_dir.clone();

            self.task_processor.spawn_task(move |ui_sender| {
                // Send an initial message to the UI.
                let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
                    "Downloading wiitdb.zip...".to_string(),
                ));

                util::wiitdb::download_and_extract_database(&base_dir)
                    .context("Failed to download and extract the database")?;

                let _ = ui_sender.send(BackgroundMessage::Info(
                    "wiitdb.zip downloaded and extracted successfully.".to_string(),
                ));

                Ok(())
            });
        }
    }

    pub fn spawn_check_for_new_version_task(&self) {
        self.task_processor.spawn_task(move |ui_sender| {
            let update_info = check_for_new_version()?;
            let _ = ui_sender.send(BackgroundMessage::GotUpdate(update_info));
            Ok(())
        });
    }

    pub fn spawn_update_apps_task(&self) {
        self.task_processor.spawn_task(move |ui_sender| {
            let _ = ui_sender.send(BackgroundMessage::UpdateStatus(
                "Updating OSCWii contents...".to_string(),
            ));

            let new_cache = oscwii::AppCache::new()?;
            let _ = ui_sender.send(BackgroundMessage::GotNewAppCache(new_cache));

            let _ = ui_sender.send(BackgroundMessage::Info(
                "OSCWii contents cached successfully.".to_string(),
            ));

            Ok(())
        });
    }
}
