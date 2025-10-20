// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    covers,
    disc_info::DiscInfo,
    extensions,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
    osc::OscApp,
    tasks::{BackgroundMessage, TaskProcessor},
    titles::Titles,
    ui,
    updater::UpdateInfo,
    util,
};
use eframe::egui;
use egui_file_dialog::FileDialog;
use egui_notify::{Anchor, Toasts};
use std::path::{Path, PathBuf};

pub struct App {
    pub data_dir: PathBuf,
    pub current_view: ui::View,
    pub config: Config,
    pub update_info: Option<UpdateInfo>,
    pub games: Vec<Game>,
    pub filtered_games: Vec<Game>,
    pub game_search: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub titles: Option<Titles>,
    pub disc_info: Option<(String, DiscInfo)>,
    pub removing_game: Option<Game>,
    pub removing_hbc_app: Option<HbcApp>,
    pub hbc_app_info: Option<HbcApp>,
    pub hbc_app_search: String,
    pub hbc_apps: Vec<HbcApp>,
    pub filtered_hbc_apps: Vec<HbcApp>,
    pub task_processor: TaskProcessor,
    pub osc_task_processor: Option<TaskProcessor>,
    pub choose_mount_point: FileDialog,
    pub choose_games: FileDialog,
    pub choose_hbc_apps: FileDialog,
    pub toasts: Toasts,
    pub osc_apps: Option<Vec<OscApp>>,
    pub status: String,
    pub osc_status: String,
    pub got_redump_db: bool,
}

impl App {
    pub fn new(data_dir: &Path) -> Self {
        let toasts = Toasts::default()
            .with_anchor(Anchor::BottomRight)
            .with_margin(egui::vec2(8.0, 8.0))
            .with_shadow(egui::Shadow {
                offset: [0, 0],
                blur: 0,
                spread: 1,
                color: egui::Color32::GRAY,
            });

        Self {
            data_dir: data_dir.to_path_buf(),
            current_view: ui::View::Games,
            config: Config::load(data_dir),
            update_info: None,
            games: Vec::new(),
            filtered_games: Vec::new(),
            game_search: String::new(),
            show_wii: true,
            show_gc: true,
            titles: None,
            disc_info: None,
            removing_game: None,
            task_processor: TaskProcessor::init(),
            osc_task_processor: Some(TaskProcessor::init()),
            choose_mount_point: FileDialog::new().as_modal(true),
            choose_games: FileDialog::new()
                .as_modal(true)
                .add_file_filter_extensions(
                    "Nintendo Optical Disc",
                    extensions::SUPPORTED_INPUT_EXTENSIONS.to_vec(),
                )
                .default_file_filter("Nintendo Optical Disc"),
            choose_hbc_apps: FileDialog::new()
                .as_modal(true)
                .add_file_filter_extensions("HBC App (zip)", vec!["zip", "ZIP"])
                .default_file_filter("HBC App (zip)"),
            toasts,
            hbc_app_search: String::new(),
            hbc_apps: Vec::new(),
            filtered_hbc_apps: Vec::new(),
            removing_hbc_app: None,
            hbc_app_info: None,
            osc_apps: None,
            status: String::new(),
            osc_status: String::new(),
            got_redump_db: false,
        }
    }

    pub fn update_filtered_games(&mut self) {
        if !self.show_wii && !self.show_gc {
            self.filtered_games.clear();
            return;
        }

        let filtered_games = self
            .games
            .iter()
            .filter(|game| (self.show_wii && game.is_wii) || (self.show_gc && !game.is_wii));

        if self.game_search.is_empty() {
            self.filtered_games = filtered_games.cloned().collect();
            return;
        }

        let game_search = self.game_search.to_lowercase();
        self.filtered_games = filtered_games
            .filter(|game| game.search_str.contains(&game_search))
            .cloned()
            .collect();
    }

    pub fn update_filtered_hbc_apps(&mut self) {
        if self.hbc_app_search.is_empty() {
            self.filtered_hbc_apps = self.hbc_apps.clone();
            return;
        }

        let hbc_app_search = self.hbc_app_search.to_lowercase();
        self.filtered_hbc_apps = self
            .hbc_apps
            .iter()
            .filter(|hbc_app| hbc_app.search_str.contains(&hbc_app_search))
            .cloned()
            .collect();
    }

    pub fn update_title(&self, ctx: &egui::Context) {
        let title = match self.current_view {
            ui::View::Games => format!(
                "{} v{} • {} Games • {} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                self.games.len(),
                self.config.get_drive_name(),
                util::get_disk_usage(&self.config.contents.mount_point).unwrap_or_default()
            ),
            ui::View::HbcApps => format!(
                "{} v{} • {} Apps • {} {}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                self.hbc_apps.len(),
                self.config.get_drive_name(),
                util::get_disk_usage(&self.config.contents.mount_point).unwrap_or_default()
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

    pub fn refresh_games(&mut self) {
        let res = games::list(&self.config.contents.mount_point, &self.titles);
        match res {
            Ok(games) => {
                self.games = games;
                games::sort(&mut self.games, &self.config.contents.sort_by);
                self.update_filtered_games();
            }
            Err(e) => {
                self.toasts.error(e.to_string());
            }
        }

        covers::spawn_download_covers_task(self);
    }

    pub fn refresh_hbc_apps(&mut self) {
        let res = hbc_apps::list(&self.config.contents.mount_point);
        match res {
            Ok(hbc_apps) => {
                self.hbc_apps = hbc_apps;
                hbc_apps::sort(&mut self.hbc_apps, &self.config.contents.sort_by);
                self.update_filtered_hbc_apps();
            }
            Err(e) => {
                self.toasts.error(e.to_string());
            }
        }
    }

    pub fn apply_sorting(&mut self) {
        games::sort(&mut self.games, &self.config.contents.sort_by);
        self.update_filtered_games();
        hbc_apps::sort(&mut self.hbc_apps, &self.config.contents.sort_by);
        self.update_filtered_hbc_apps();
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::root::update(ctx, self);

        // Process background messages (from task_processor)
        while let Ok(msg) = self.task_processor.msg_receiver.try_recv() {
            match msg {
                BackgroundMessage::NotifyInfo(string) => {
                    self.toasts.info(string);
                }
                BackgroundMessage::NotifyError(string) => {
                    self.toasts.error(string);
                }
                BackgroundMessage::UpdateStatus(string) => {
                    self.status = string;
                }
                BackgroundMessage::ClearStatus => {
                    self.status.clear();
                }
                BackgroundMessage::TriggerRefreshImages => {
                    ctx.forget_all_images();
                }
                BackgroundMessage::TriggerRefreshGames => {
                    self.refresh_games();
                }
                BackgroundMessage::TriggerRefreshHbcApps => {
                    self.refresh_hbc_apps();
                }
                BackgroundMessage::GotUpdateInfo(update_info) => {
                    self.update_info = Some(update_info);
                }
                BackgroundMessage::GotTitles(titles) => {
                    self.titles = Some(titles);
                    self.refresh_games();
                }
                BackgroundMessage::UpdateOscStatus(_)
                | BackgroundMessage::GotOscApps(_)
                | BackgroundMessage::GotRedumpDb => {}
            }

            ctx.request_repaint();
        }

        // Osc task processor
        let mut drop_task_processor = false;
        if let Some(osc_task_processor) = &self.osc_task_processor {
            while let Ok(msg) = osc_task_processor.msg_receiver.try_recv() {
                match msg {
                    BackgroundMessage::UpdateOscStatus(string) => {
                        self.osc_status = string;
                    }
                    BackgroundMessage::NotifyInfo(string) => {
                        self.toasts.info(string);
                    }
                    BackgroundMessage::GotOscApps(osc_apps) => {
                        self.osc_apps = Some(osc_apps);
                        drop_task_processor = true;
                    }
                    BackgroundMessage::GotRedumpDb => {
                        self.got_redump_db = true;
                    }
                    _ => {}
                }
            }

            ctx.request_repaint();
        }

        if drop_task_processor {
            self.osc_task_processor = None;
        }
    }
}
