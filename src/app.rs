// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{ArchiveFormat, Config},
    covers,
    disc_info::DiscInfo,
    extensions,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
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
use egui_notify::{Anchor, Toasts};
use std::{
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

pub type GameInfoData = (Game, Result<DiscInfo, String>, Result<GameInfo, String>);

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
    pub choose_hbc_apps: FileDialog,
    pub choose_archive_path: FileDialog,
    pub choose_file_to_push: FileDialog,
    pub archiving_game: Option<PathBuf>,
    pub toasts: Toasts,
    pub osc_apps: Option<Vec<OscApp>>,
    pub filtered_osc_apps: Vec<OscApp>,
    pub osc_app_search: String,
    pub status: String,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub is_info_open: bool,
}

impl App {
    pub fn new(data_dir: &Path) -> Self {
        let config = Config::load(data_dir);

        let choose_archive_path = FileDialog::new()
            .as_modal(true)
            .add_save_extension(ArchiveFormat::Rvz.as_ref(), "rvz")
            .add_save_extension(ArchiveFormat::Iso.as_ref(), "iso")
            .default_save_extension(config.contents.archive_format.as_ref());

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
            config,
            update_info: None,
            games: Vec::new(),
            filtered_games: Vec::new(),
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
            toasts,
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
                self.toasts.error(e.to_string());
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
                self.toasts.error(e.to_string());
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
                BackgroundMessage::NotifyInfo(string) => {
                    self.toasts.info(string);
                }
                BackgroundMessage::NotifyError(string) => {
                    self.toasts.error(string).duration(Duration::from_secs(10));
                }
                BackgroundMessage::NotifySuccess(string) => {
                    self.toasts
                        .success(string)
                        .duration(Duration::from_secs(10));
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
                        self.toasts.error(e.to_string());
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
