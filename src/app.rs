// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{ArchiveFormat, Config, SortBy},
    covers,
    disc_info::DiscInfo,
    extensions,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
    notifications::Notifications,
    osc::{self, OscApp},
    tasks::{BackgroundMessage, TaskProcessor},
    titles::Titles,
    ui::{self, Modal},
    updater::UpdateInfo,
    util,
    wiitdb::{self, GameInfo},
};
use crossbeam_channel::{Receiver, unbounded};
use eframe::egui;
use egui_file_dialog::FileDialog;
use size::Size;
use smallvec::SmallVec;
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
    pub games: Box<[Game]>,
    pub filtered_games: SmallVec<[u16; 512]>,
    pub filtered_wii_games: SmallVec<[u16; 256]>,
    pub filtered_wii_games_size: Size,
    pub filtered_gc_games: SmallVec<[u16; 256]>,
    pub filtered_gc_games_size: Size,
    pub game_search: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub titles: Option<Titles>,
    pub hbc_app_search: String,
    pub hbc_apps: Box<[HbcApp]>,
    pub filtered_hbc_apps: SmallVec<[u16; 64]>,
    pub filtered_hbc_apps_size: Size,
    pub task_processor: TaskProcessor,
    pub downloading_osc_icons: Option<Receiver<String>>,
    pub choose_mount_point: FileDialog,
    pub choose_games: FileDialog,
    pub choose_hbc_apps: FileDialog,
    pub choose_archive_path: FileDialog,
    pub choose_file_to_push: FileDialog,
    pub archiving_game: Option<PathBuf>,
    pub osc_apps: Box<[OscApp]>,
    pub filtered_osc_apps: SmallVec<[u16; 512]>,
    pub osc_app_search: String,
    pub status: String,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub notifications: Notifications,
    pub current_modal: Modal,
    pub discs_to_convert: Box<[DiscInfo]>,
    pub current_game_info: Option<GameInfoData>,
}

impl App {
    pub fn new(data_dir: &Path) -> Self {
        let config = Config::load(data_dir);

        let choose_archive_path = FileDialog::new()
            .as_modal(true)
            .add_save_extension(ArchiveFormat::Rvz.as_str(), ArchiveFormat::Rvz.extension())
            .add_save_extension(ArchiveFormat::Iso.as_str(), ArchiveFormat::Iso.extension())
            .default_save_extension(config.contents.archive_format.as_str());

        Self {
            data_dir: data_dir.to_path_buf(),
            current_view: ui::View::Games,
            config,
            update_info: None,
            games: Box::new([]),
            filtered_games: SmallVec::new(),
            filtered_wii_games: SmallVec::new(),
            filtered_gc_games: SmallVec::new(),
            filtered_wii_games_size: Size::from_bytes(0),
            filtered_gc_games_size: Size::from_bytes(0),
            game_search: String::new(),
            show_wii: true,
            show_gc: true,
            titles: None,
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
            hbc_app_search: String::new(),
            hbc_apps: Box::new([]),
            filtered_hbc_apps: SmallVec::new(),
            filtered_hbc_apps_size: Size::from_bytes(0),
            osc_apps: Box::new([]),
            filtered_osc_apps: SmallVec::new(),
            osc_app_search: String::new(),
            status: String::new(),
            wiitdb: None,
            notifications: Notifications::new(),
            current_modal: Modal::None,
            discs_to_convert: Box::new([]),
            current_game_info: None,
        }
    }

    pub fn change_view(&mut self, ctx: &egui::Context, view: ui::View) {
        if self.current_view == view {
            return;
        }

        // drop the osc icons from memory
        if self.current_view == ui::View::Osc {
            for osc_app in &self.osc_apps {
                ctx.forget_image(&osc_app.icon_uri)
            }
        }

        self.current_view = view;
        self.update_title(ctx);
    }

    pub fn update_filtered_games(&mut self) {
        self.filtered_wii_games_size = Size::from_bytes(0);
        self.filtered_gc_games_size = Size::from_bytes(0);

        self.filtered_games.clear();
        self.filtered_wii_games.clear();
        self.filtered_gc_games.clear();

        if self.game_search.is_empty() {
            self.filtered_games.extend(0..self.games.len() as u16);

            for (i, game) in self.games.iter().enumerate() {
                if game.is_wii {
                    self.filtered_wii_games.push(i as u16);
                    self.filtered_wii_games_size += game.size;
                } else {
                    self.filtered_gc_games.push(i as u16);
                    self.filtered_gc_games_size += game.size;
                }
            }
        } else {
            let game_search = self.game_search.to_lowercase();

            for (i, game) in self.games.iter().enumerate() {
                if game.search_str.contains(&game_search) {
                    let i = i as u16;

                    self.filtered_games.push(i);

                    if game.is_wii {
                        self.filtered_wii_games.push(i);
                        self.filtered_wii_games_size += game.size;
                    } else {
                        self.filtered_gc_games.push(i);
                        self.filtered_gc_games_size += game.size;
                    }
                }
            }
        };
    }

    pub fn update_filtered_hbc_apps(&mut self) {
        self.filtered_hbc_apps.clear();
        self.filtered_hbc_apps_size = Size::from_bytes(0);

        if self.hbc_app_search.is_empty() {
            self.filtered_hbc_apps.extend(0..self.hbc_apps.len() as u16);

            for hbc_app in &self.hbc_apps {
                self.filtered_hbc_apps_size += hbc_app.size;
            }

            return;
        }

        let hbc_app_search = self.hbc_app_search.to_lowercase();
        for (i, hbc_app) in self.hbc_apps.iter().enumerate() {
            if hbc_app.search_str.contains(&hbc_app_search) {
                self.filtered_hbc_apps.push(i as u16);
                self.filtered_hbc_apps_size += hbc_app.size;
            }
        }
    }

    pub fn update_filtered_osc_apps(&mut self) {
        self.filtered_osc_apps.clear();

        if self.osc_app_search.is_empty() {
            self.filtered_osc_apps.extend(0..self.osc_apps.len() as u16);
            return;
        }

        let osc_app_search = self.osc_app_search.to_lowercase();
        for (i, osc_app) in self.osc_apps.iter().enumerate() {
            if osc_app.search_str.contains(&osc_app_search) {
                self.filtered_osc_apps.push(i as u16);
            }
        }
    }

    pub fn update_title(&self, ctx: &egui::Context) {
        let title = format!(
            "{} • {} • {} ({})",
            env!("CARGO_PKG_NAME"),
            self.current_view.title(),
            self.config.get_drive_path_str(),
            util::get_disk_usage(&self.config.contents.mount_point)
        );

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    pub fn refresh_games(&mut self, ctx: &egui::Context) {
        self.games = games::list(&self.config.contents.mount_point, &self.titles);

        games::sort(&mut self.games, SortBy::None, self.config.contents.sort_by);
        self.update_filtered_games();

        self.update_title(ctx);
        covers::spawn_download_covers_task(self);
    }

    pub fn refresh_hbc_apps(&mut self, ctx: &egui::Context) {
        self.hbc_apps = hbc_apps::list(&self.config.contents.mount_point);

        hbc_apps::sort(
            &mut self.hbc_apps,
            SortBy::None,
            self.config.contents.sort_by,
        );
        self.update_filtered_hbc_apps();

        self.update_title(ctx);
    }

    pub fn apply_sorting(&mut self, sort_by: SortBy) {
        games::sort(&mut self.games, self.config.contents.sort_by, sort_by);
        hbc_apps::sort(&mut self.hbc_apps, self.config.contents.sort_by, sort_by);

        self.config.contents.sort_by = sort_by;

        if let Err(e) = self.config.write() {
            self.notifications.show_err(e);
        }

        self.update_filtered_games();
        self.update_filtered_hbc_apps();
    }

    pub fn check_for_hbc_app_updates(&mut self) {
        for hbc_app in self.hbc_apps.iter_mut() {
            if let Ok(osc_app_i) = self
                .osc_apps
                .binary_search_by(|a| a.meta.name.cmp(&hbc_app.meta.name))
            {
                hbc_app.osc_app_i = Some(osc_app_i);
            }
        }
    }

    pub fn download_osc_icons(&mut self) {
        let icons_dir = self.data_dir.join("osc-icons");

        let (sender, receiver) = unbounded();

        let osc_apps = self.osc_apps.clone();
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

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::root::update(ctx, self);

        // Process background messages (from task_processor)
        let mut should_repaint = false;
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
                    self.osc_apps = osc_apps;
                    self.update_filtered_osc_apps();
                    self.check_for_hbc_app_updates();
                }
                BackgroundMessage::SetArchiveFormat(format) => {
                    self.config.contents.archive_format = format;
                    if let Err(e) = self.config.write() {
                        self.notifications.show_err(e);
                    }
                }
            }

            should_repaint = true;
        }

        if should_repaint {
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
