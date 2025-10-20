// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    disc_info::DiscInfo,
    games::{self, Game},
    tasks::TaskProcessor,
    titles::Titles,
    ui,
    updater::{self, UpdateInfo},
};
use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_notify::{Anchor, Toasts};
use parking_lot::Mutex;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct App {
    pub data_dir: PathBuf,
    pub current_view: ui::View,
    pub config: Config,
    pub update_info: Arc<Mutex<Option<UpdateInfo>>>,
    pub games: Arc<Mutex<Vec<Game>>>,
    pub shown_games: Arc<Mutex<Vec<Game>>>,
    pub game_search: String,
    pub titles: Arc<Mutex<Option<Titles>>>,
    pub disc_info: Option<DiscInfo>,
    pub removing_game: Option<Game>,
    pub task_processor: TaskProcessor,
    pub choose_mount_point: FileDialog,
    pub toasts: Arc<Mutex<Toasts>>,

    // Pending actions to perform after the UI has been updated
    pub pending_refresh_images: Arc<Mutex<bool>>,
}

impl App {
    pub fn new(data_dir: &Path) -> Self {
        let config = Config::load(data_dir);

        let toasts = Arc::new(Mutex::new(
            Toasts::default()
                .with_anchor(Anchor::BottomRight)
                .with_margin(egui::vec2(8.0, 8.0))
                .with_shadow(egui::Shadow {
                    offset: [0, 0],
                    blur: 0,
                    spread: 1,
                    color: egui::Color32::GRAY,
                }),
        ));

        let task_processor = TaskProcessor::init(toasts.clone());

        Self {
            data_dir: data_dir.to_path_buf(),
            current_view: ui::View::Games,
            config,
            update_info: Arc::new(Mutex::new(None)),
            games: Arc::new(Mutex::new(Vec::new())),
            shown_games: Arc::new(Mutex::new(Vec::new())),
            game_search: String::new(),
            titles: Arc::new(Mutex::new(None)),
            disc_info: None,
            removing_game: None,
            task_processor,
            choose_mount_point: FileDialog::new().as_modal(true),
            toasts,
            pending_refresh_images: Arc::new(Mutex::new(false)),
        }
    }

    pub fn spawn_get_titles_task(&self) {
        let data_dir = self.data_dir.clone();
        let titles = self.titles.clone();

        self.task_processor.spawn(move |status, toasts| {
            *status.lock() = "📓 Loading titles...".to_string();

            let new_titles = Titles::load(&data_dir)?;
            *titles.lock() = Some(new_titles);
            toasts.lock().info("📓 Titles loaded".to_string());

            Ok(())
        });
    }

    pub fn spawn_get_games_task(&self) {
        let mount_point = self.config.contents.mount_point.clone();
        let titles = self.titles.clone();
        let games = self.games.clone();
        let shown_games = self.shown_games.clone();
        let pending_refresh_images = self.pending_refresh_images.clone();

        self.task_processor.spawn(move |status, toasts| {
            *status.lock() = "🎮 Loading games...".to_string();

            let new_games = games::list(&mount_point, &titles.lock())?;
            *games.lock() = new_games.clone();
            *shown_games.lock() = new_games;

            toasts.lock().info("🎮 Games loaded".to_string());

            *pending_refresh_images.lock() = true;

            Ok(())
        });
    }

    pub fn spawn_check_update_task(&self) {
        let update_info = self.update_info.clone();

        self.task_processor.spawn(move |status, toasts| {
            *status.lock() = "✈ Checking for updates...".to_string();

            let new_update_info = updater::check()?;

            if let Some(update_info) = &new_update_info {
                toasts.lock().info(update_info.to_string());
            }

            *update_info.lock() = new_update_info;

            Ok(())
        });
    }

    pub fn apply_pending(&mut self, ctx: &egui::Context) {
        let mut pending_refresh_images = self.pending_refresh_images.lock();
        if *pending_refresh_images {
            ctx.forget_all_images();
            *pending_refresh_images = false;
        }
    }
}
