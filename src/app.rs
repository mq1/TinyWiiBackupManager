// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use std::path::PathBuf;

use anyhow::{Error, Result};
use eframe::egui;
use egui_inbox::UiInbox;
use egui_suspense::EguiSuspense;

use crate::base_dir::BaseDir;
use crate::messages::{BackgroundMessage, handle_messages};
use crate::{
    components::{self, console_filter::ConsoleFilter, update_notifier::UpdateInfo},
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
    pub(crate) inbox: UiInbox<BackgroundMessage>,
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
    /// Toasts
    pub top_left_toasts: egui_notify::Toasts,
    pub toasts: egui_notify::Toasts,
}

impl App {
    /// Initializes the application with the specified WBFS directory.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Install image loaders
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Initialize the update checker based on the TWBM_DISABLE_UPDATES env var
        let update_checker = std::env::var_os("TWBM_DISABLE_UPDATES")
            .is_none()
            .then(|| EguiSuspense::single_try(components::update_notifier::check_for_new_version));

        // Pretty notifications
        let toasts = egui_notify::Toasts::default()
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

        // Show toast to choose base dir
        top_left_toasts
            .custom(
                "  Click on \"📄 File\" to select a Drive/Directory",
                "⬆📄".to_string(),
                egui::Color32::DARK_GRAY,
            )
            .closable(false)
            .duration(None);

        Self {
            update_checker,
            top_left_toasts,
            toasts,
            ..Default::default()
        }
    }

    pub fn choose_base_dir(&mut self) -> Result<()> {
        let new_dir = rfd::FileDialog::new()
            .set_title("Select New Base Directory")
            .pick_folder();

        if let Some(new_dir) = new_dir {
            let base_dir = BaseDir::new(new_dir)?;

            let sender = self.inbox.sender();
            let watcher = base_dir.get_watcher(move |_res| {
                let _ = sender.send(BackgroundMessage::DirectoryChanged);
            })?;

            self.watcher = Some(watcher);
            self.base_dir = Some(base_dir);

            self.refresh_games()?;
        }

        Ok(())
    }

    fn refresh_games(&mut self) -> Result<()> {
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

        if let Some(paths) = paths.filter(|p| !p.is_empty()) {
            self.spawn_conversion_worker(paths);
        }
    }

    /// Converts ISO files to WBFS in a background thread
    fn spawn_conversion_worker(&mut self, paths: Vec<PathBuf>) {
        let base_dir = self.base_dir.clone().unwrap().path().to_path_buf();
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
        handle_messages(self, ctx);

        match self.conversion_state {
            ConversionState::Converting { .. } => ctx.set_cursor_icon(egui::CursorIcon::Wait),
            _ => ctx.set_cursor_icon(egui::CursorIcon::Default),
        }

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
                components::game_info::ui_game_info_window(
                    ctx,
                    game,
                    &mut is_open,
                    self.inbox.sender(),
                );
                is_open
            })
        });

        self.top_left_toasts.show(ctx);
        self.toasts.show(ctx);
    }
}
