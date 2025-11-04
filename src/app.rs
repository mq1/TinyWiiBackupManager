// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{ArchiveFormat, Config},
    covers,
    disc_info::DiscInfo,
    extensions,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
    notifications::Notifications,
    osc::{self, OscApp},
    tasks::{BackgroundMessage, TaskProcessor},
    titles::Titles,
    ui,
    updater::UpdateInfo,
    util,
    wiitdb::{self, GameInfo},
};
use crossbeam_channel::{Receiver, unbounded};
use eframe::egui;
use egui_file_dialog::FileDialog;
use size::Size;
use std::{
    path::{Path, PathBuf},
    thread,
};

pub type GameInfoData = (Game, Result<DiscInfo, String>, Result<GameInfo, String>);

pub struct App {
    pub data_dir: PathBuf,
    pub current_view: ui::View,
    pub config: Config,
    pub update_info: Option<UpdateInfo>,
    pub games: Vec<Game>,
    pub filtered_games: Vec<Game>,
    pub filtered_wii_games_len: usize,
    pub filtered_wii_games_size: Size,
    pub filtered_gc_games_len: usize,
    pub filtered_gc_games_size: Size,
    pub game_search: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub titles: Option<Titles>,
    pub game_info: Option<GameInfoData>,
    pub deleting_game: Option<Game>,
    pub deleting_hbc_app: Option<HbcApp>,
    pub hbc_app_info: Option<HbcApp>,
    pub hbc_app_search: String,
    pub hbc_apps: Vec<HbcApp>,
    pub filtered_hbc_apps: Vec<HbcApp>,
    pub task_processor: TaskProcessor,
    pub downloading_osc_icons: Option<Receiver<String>>,
    pub choose_mount_point: FileDialog,
    pub choose_games: FileDialog,
    pub choosing_games: Vec<DiscInfo>,
    pub choose_hbc_apps: FileDialog,
    pub choose_archive_path: FileDialog,
    pub choose_file_to_push: FileDialog,
    pub archiving_game: Option<PathBuf>,
    pub osc_apps: Option<Vec<OscApp>>,
    pub filtered_osc_apps: Vec<OscApp>,
    pub osc_app_search: String,
    pub status: String,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub is_info_open: bool,
    pub notifications: Notifications,
}

impl App {
    pub fn new(data_dir: &Path) -> Self {
        let config = Config::load(data_dir);

        let choose_archive_path = FileDialog::new()
            .as_modal(true)
            .add_save_extension(ArchiveFormat::Rvz.as_ref(), "rvz")
            .add_save_extension(ArchiveFormat::Iso.as_ref(), "iso")
            .default_save_extension(config.contents.archive_format.as_ref());

        Self {
            data_dir: data_dir.to_path_buf(),
            current_view: ui::View::Games,
            config,
            update_info: None,
            games: Vec::new(),
            filtered_games: Vec::new(),
            filtered_wii_games_len: 0,
            filtered_wii_games_size: Size::from_bytes(0),
            filtered_gc_games_len: 0,
            filtered_gc_games_size: Size::from_bytes(0),
            game_search: String::new(),
            show_wii: true,
            show_gc: true,
            titles: None,
            game_info: None,
            deleting_game: None,
            task_processor: TaskProcessor::init(),
            downloading_osc_icons: None,
            choose_mount_point: FileDialog::new().as_modal(true),
            choose_games: FileDialog::new()
                .as_modal(true)
                .add_file_filter_extensions(
                    "Nintendo Optical Disc",
                    extensions::SUPPORTED_INPUT_EXTENSIONS.to_vec(),
                )
                .default_file_filter("Nintendo Optical Disc"),
            choosing_games: Vec::new(),
            choose_hbc_apps: FileDialog::new()
                .as_modal(true)
                .add_file_filter_extensions("HBC App (zip)", vec!["zip", "ZIP"])
                .default_file_filter("HBC App (zip)"),
            choose_file_to_push: FileDialog::new()
                .as_modal(true)
                .add_file_filter_extensions("HBC App (zip/dol/elf)", vec!["zip", "dol", "elf"])
                .default_file_filter("HBC App (zip/dol/elf)"),
            choose_archive_path,
            archiving_game: None,
            hbc_app_search: String::new(),
            hbc_apps: Vec::new(),
            filtered_hbc_apps: Vec::new(),
            deleting_hbc_app: None,
            hbc_app_info: None,
            osc_apps: None,
            filtered_osc_apps: Vec::new(),
            osc_app_search: String::new(),
            status: String::new(),
            wiitdb: None,
            is_info_open: false,
            notifications: Notifications::new(),
        }
    }

    pub fn update_filtered_games(&mut self) {
        if self.game_search.is_empty() {
            self.filtered_games.clone_from(&self.games);
        } else {
            let game_search = self.game_search.to_lowercase();

            self.filtered_games = self
                .games
                .iter()
                .filter(|game| game.search_str.contains(&game_search))
                .cloned()
                .collect();
        }

        self.filtered_wii_games_len = 0;
        self.filtered_gc_games_len = 0;

        self.filtered_wii_games_size = Size::from_bytes(0);
        self.filtered_gc_games_size = Size::from_bytes(0);

        for game in &self.filtered_games {
            if game.is_wii {
                self.filtered_wii_games_len += 1;
                self.filtered_wii_games_size += game.size;
            } else {
                self.filtered_gc_games_len += 1;
                self.filtered_gc_games_size += game.size;
            }
        }
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

    pub fn update_filtered_osc_apps(&mut self) {
        if let Some(osc_apps) = &self.osc_apps {
            if self.osc_app_search.is_empty() {
                self.filtered_osc_apps = osc_apps.clone();
                return;
            }

            let osc_app_search = self.osc_app_search.to_lowercase();
            self.filtered_osc_apps = osc_apps
                .iter()
                .filter(|osc_app| osc_app.search_str.contains(&osc_app_search))
                .cloned()
                .collect();
        }
    }

    pub fn update_title(&self, ctx: &egui::Context) {
        let title = match self.current_view {
            ui::View::Games => format!(
                "{} • {} Games in {} {}",
                env!("CARGO_PKG_NAME"),
                self.games.len(),
                self.config.get_drive_path_str(),
                util::get_disk_usage(&self.config.contents.mount_point).unwrap_or_default()
            ),
            ui::View::HbcApps => format!(
                "{} • {} Apps in {} {}",
                env!("CARGO_PKG_NAME"),
                self.hbc_apps.len(),
                self.config.get_drive_path_str(),
                util::get_disk_usage(&self.config.contents.mount_point).unwrap_or_default()
            ),
            ui::View::Osc => format!("{} • Open Shop Channel", env!("CARGO_PKG_NAME"),),
            ui::View::Tools => format!("{} • Tools", env!("CARGO_PKG_NAME"),),
            ui::View::Settings => format!("{} • Settings", env!("CARGO_PKG_NAME"),),
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    pub fn refresh_games(&mut self, ctx: &egui::Context) {
        let res = games::list(&self.config.contents.mount_point, &self.titles);
        match res {
            Ok(games) => {
                self.games = games;
                games::sort(&mut self.games, &self.config.contents.sort_by);
                self.update_filtered_games();
            }
            Err(e) => {
                self.notifications.show_err(e);
            }
        }

        self.update_title(ctx);
        covers::spawn_download_covers_task(self);
    }

    pub fn refresh_hbc_apps(&mut self, ctx: &egui::Context) {
        let res = hbc_apps::list(&self.config.contents.mount_point);
        match res {
            Ok(hbc_apps) => {
                self.hbc_apps = hbc_apps;
                hbc_apps::sort(&mut self.hbc_apps, &self.config.contents.sort_by);
                self.update_filtered_hbc_apps();
            }
            Err(e) => {
                self.notifications.show_err(e);
            }
        }

        self.update_title(ctx);
    }

    pub fn apply_sorting(&mut self) {
        games::sort(&mut self.games, &self.config.contents.sort_by);
        self.update_filtered_games();

        hbc_apps::sort(&mut self.hbc_apps, &self.config.contents.sort_by);
        self.update_filtered_hbc_apps();
    }

    pub fn download_osc_icons(&mut self) {
        let icons_dir = self.data_dir.join("osc-icons");

        if let Some(osc_apps) = self.osc_apps.clone() {
            let (sender, receiver) = unbounded();

            thread::spawn(move || {
                for osc_app in osc_apps {
                    if osc::download_icon(&osc_app.meta, &icons_dir).is_ok() {
                        let _ = sender.send(osc_app.icon_uri);
                    }
                }
            });

            self.downloading_osc_icons = Some(receiver);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::root::update(ctx, self);

        // Process background messages (from task_processor)
        while let Ok(msg) = self.task_processor.msg_receiver.try_recv() {
            match msg {
                BackgroundMessage::NotifyInfo(i) => {
                    self.notifications.show_info(&i);
                }
                BackgroundMessage::NotifyError(e) => {
                    self.notifications.show_err(e);
                }
                BackgroundMessage::NotifySuccess(s) => {
                    self.notifications.show_success(&s);
                }
                BackgroundMessage::UpdateStatus(string) => {
                    self.status = string;
                }
                BackgroundMessage::ClearStatus => {
                    self.status.clear();
                }
                BackgroundMessage::TriggerRefreshImage(uri) => {
                    ctx.forget_image(&uri);
                }
                BackgroundMessage::TriggerRefreshGames => {
                    self.refresh_games(ctx);
                }
                BackgroundMessage::TriggerRefreshHbcApps => {
                    self.refresh_hbc_apps(ctx);
                }
                BackgroundMessage::GotUpdateInfo(update_info) => {
                    self.update_info = Some(update_info);
                }
                BackgroundMessage::GotTitles(titles) => {
                    self.titles = Some(titles);
                    self.refresh_games(ctx);
                }
                BackgroundMessage::GotOscApps(osc_apps) => {
                    self.osc_apps = Some(osc_apps);
                    self.update_filtered_osc_apps();
                }
                BackgroundMessage::SetArchiveFormat(format) => {
                    self.config.contents.archive_format = format;
                    if let Err(e) = self.config.write() {
                        self.notifications.show_err(e);
                    }
                }
            }

            ctx.request_repaint();
        }

        // Process OSC icon updates
        if let Some(receiver) = self.downloading_osc_icons.as_mut() {
            while let Ok(icon_uri) = receiver.try_recv() {
                ctx.forget_image(&icon_uri);
            }
        }
    }
}
