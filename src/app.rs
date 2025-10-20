// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    disc_info::DiscInfo,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
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
    pub filtered_games: Arc<Mutex<Vec<Game>>>,
    pub game_search: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub titles: Arc<Mutex<Option<Titles>>>,
    pub disc_info: Option<DiscInfo>,
    pub removing_game: Option<Game>,
    pub removing_hbc_app: Option<HbcApp>,
    pub hbc_app_info: Option<HbcApp>,
    pub hbc_app_search: String,
    pub hbc_apps: Arc<Mutex<Vec<HbcApp>>>,
    pub filtered_hbc_apps: Arc<Mutex<Vec<HbcApp>>>,
    pub task_processor: TaskProcessor,
    pub choose_mount_point: FileDialog,
    pub toasts: Arc<Mutex<Toasts>>,

    // Pending actions to perform after the UI has been updated
    pub pending_refresh_images: Arc<Mutex<bool>>,
    pub pending_update_title: Arc<Mutex<bool>>,
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
            filtered_games: Arc::new(Mutex::new(Vec::new())),
            game_search: String::new(),
            show_wii: true,
            show_gc: true,
            titles: Arc::new(Mutex::new(None)),
            disc_info: None,
            removing_game: None,
            task_processor,
            choose_mount_point: FileDialog::new().as_modal(true),
            toasts,
            hbc_app_search: String::new(),
            hbc_apps: Arc::new(Mutex::new(Vec::new())),
            filtered_hbc_apps: Arc::new(Mutex::new(Vec::new())),
            removing_hbc_app: None,
            hbc_app_info: None,
            pending_refresh_images: Arc::new(Mutex::new(false)),
            pending_update_title: Arc::new(Mutex::new(false)),
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
        let pending_update_title = self.pending_update_title.clone();
        let filtered_games = self.filtered_games.clone();

        self.task_processor.spawn(move |status, toasts| {
            *status.lock() = "🎮 Loading games...".to_string();

            let new_games = games::list(&mount_point, &titles.lock())?;
            *games.lock() = new_games.clone();
            *filtered_games.lock() = new_games;

            toasts.lock().info("🎮 Games loaded".to_string());

            *pending_update_title.lock() = true;

            Ok(())
        });
    }

    pub fn spawn_get_hbc_apps_task(&self) {
        let mount_point = self.config.contents.mount_point.clone();
        let hbc_apps = self.hbc_apps.clone();
        let filtered_hbc_apps = self.filtered_hbc_apps.clone();

        self.task_processor.spawn(move |status, toasts| {
            *status.lock() = "🎮 Loading HBC apps...".to_string();

            let new_hbc_apps = hbc_apps::list(&mount_point)?;
            *hbc_apps.lock() = new_hbc_apps.clone();
            *filtered_hbc_apps.lock() = new_hbc_apps;

            toasts.lock().info("🎮 HBC apps loaded".to_string());

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

    pub fn update_filtered_games(&mut self) {
        let mut filtered_games = self.filtered_games.lock();

        if !self.show_wii && !self.show_gc {
            filtered_games.clear();
            return;
        }

        *filtered_games = self
            .games
            .lock()
            .iter()
            .filter(|game| (self.show_wii && game.is_wii) || (self.show_gc && !game.is_wii))
            .cloned()
            .collect();

        if self.game_search.is_empty() {
            return;
        }

        let game_search = self.game_search.to_lowercase();
        filtered_games.retain(|game| game.search_str.contains(&game_search));
    }

    pub fn update_filtered_hbc_apps(&mut self) {
        let mut filtered_hbc_apps = self.filtered_hbc_apps.lock();

        if self.hbc_app_search.is_empty() {
            return;
        }

        let hbc_app_search = self.hbc_app_search.to_lowercase();
        *filtered_hbc_apps = self
            .hbc_apps
            .lock()
            .iter()
            .filter(|hbc_app| hbc_app.search_str.contains(&hbc_app_search))
            .cloned()
            .collect();
    }

    pub fn apply_sorting(&mut self) {
        match self.config.contents.sort_by {
            SortBy::NameAscending => {
                self.filtered_games
                    .lock()
                    .sort_by(|a, b| a.display_title.cmp(&b.display_title));

                self.filtered_hbc_apps
                    .lock()
                    .sort_by(|a, b| a.name.cmp(&b.name));
            }

            SortBy::NameDescending => {
                self.filtered_games
                    .lock()
                    .sort_by(|a, b| b.display_title.cmp(&a.display_title));

                self.filtered_hbc_apps
                    .lock()
                    .sort_by(|a, b| b.name.cmp(&a.name));
            }

            SortBy::SizeAscending => {
                self.filtered_games
                    .lock()
                    .sort_by(|a, b| a.size.cmp(&b.size));

                self.filtered_hbc_apps
                    .lock()
                    .sort_by(|a, b| a.size.cmp(&b.size));
            }

            SortBy::SizeDescending => {
                self.filtered_games
                    .lock()
                    .sort_by(|a, b| b.size.cmp(&a.size));

                self.filtered_hbc_apps
                    .lock()
                    .sort_by(|a, b| b.size.cmp(&a.size));
            }
        }
    }

    pub fn update_title(&self, ctx: &egui::Context) {
        let title = match self.current_view {
            ui::View::Games => format!(
                "{} v{} • {} Games in {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                self.games.lock().len(),
                self.config.contents.mount_point.display()
            ),
            ui::View::HbcApps => format!(
                "{} v{} • {} Apps in {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                self.hbc_apps.lock().len(),
                self.config.contents.mount_point.display()
            ),
            ui::View::Osc => format!(
                "{} v{} • OSC",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ),
            ui::View::Settings => format!(
                "{} v{} • Settings",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ),
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    pub fn apply_pending(&mut self, ctx: &egui::Context) {
        let mut pending_update_title = self.pending_update_title.lock();
        if *pending_update_title {
            self.update_title(ctx);
            *pending_update_title = false;
        }

        let mut pending_refresh_images = self.pending_refresh_images.lock();
        if *pending_refresh_images {
            ctx.forget_all_images();
            *pending_refresh_images = false;
        }
    }
}
