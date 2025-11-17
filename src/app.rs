// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    archive,
    config::{ArchiveFormat, Config, SortBy},
    covers, dir_layout,
    disc_info::DiscInfo,
    extensions,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
    known_mount_points,
    notifications::Notifications,
    osc::{self, OscApp},
    tasks::{BackgroundMessage, TaskProcessor},
    titles::Titles,
    ui::{self, Modal, UiAction},
    updater::UpdateInfo,
    util, wiiload,
    wiitdb::{self, GameInfo},
};
use anyhow::Result;
use crossbeam_channel::{Receiver, unbounded};
use eframe::egui;
use egui_file_dialog::FileDialog;
use size::Size;
use smallvec::SmallVec;
use std::{fs, path::PathBuf, thread};

pub struct AppWrapper {
    pub state: AppState,
    pub ui_buffers: UiBuffers,
}

impl AppWrapper {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        Self {
            state: AppState::new(data_dir),
            ui_buffers: UiBuffers::new(config),
        }
    }
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub task_processor: TaskProcessor,
    pub titles: Option<Titles>,
    pub downloading_osc_icons: Option<Receiver<String>>,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub games: Vec<Game>,
    pub osc_apps: Box<[OscApp]>,
    pub filtered_games: SmallVec<[u16; 512]>,
    pub filtered_wii_games: SmallVec<[u16; 256]>,
    pub filtered_wii_games_size: Size,
    pub filtered_gc_games: SmallVec<[u16; 256]>,
    pub filtered_gc_games_size: Size,
    pub filtered_osc_apps: SmallVec<[u16; 512]>,
    pub filtered_hbc_apps: SmallVec<[u16; 64]>,
    pub filtered_hbc_apps_size: Size,
    pub hbc_apps: Vec<HbcApp>,
    pub current_view: ui::View,
    pub update_info: Option<UpdateInfo>,
    pub status: String,
    pub current_modal: Option<Modal>,
    pub prev_sort_by: SortBy,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            data_dir,
            current_view: ui::View::Games,
            update_info: None,
            games: Vec::new(),
            filtered_games: SmallVec::new(),
            filtered_wii_games: SmallVec::new(),
            filtered_gc_games: SmallVec::new(),
            filtered_wii_games_size: Size::from_bytes(0),
            filtered_gc_games_size: Size::from_bytes(0),
            titles: None,
            task_processor: TaskProcessor::init(),
            downloading_osc_icons: None,
            hbc_apps: Vec::new(),
            filtered_hbc_apps: SmallVec::new(),
            filtered_hbc_apps_size: Size::from_bytes(0),
            osc_apps: Box::new([]),
            filtered_osc_apps: SmallVec::new(),
            status: String::new(),
            wiitdb: None,
            current_modal: None,
            prev_sort_by: SortBy::None,
        }
    }

    pub fn check_for_hbc_app_updates(&mut self) {
        for hbc_app in self.hbc_apps.iter_mut() {
            if let Some(osc_app_i) = self
                .osc_apps
                .iter()
                .position(|osc_app| hbc_app.meta.name == osc_app.meta.name)
            {
                hbc_app.osc_app_i = Some(osc_app_i as u16);
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

    pub fn get_game_info(&self, game_id: [u8; 6]) -> Option<GameInfo> {
        self.wiitdb
            .as_ref()
            .and_then(|db| db.lookup(game_id))
            .cloned()
    }

    pub fn open_data_dir(&self) -> Result<()> {
        open::that(&self.data_dir).map_err(Into::into)
    }

    pub fn open_wiki(&self) -> Result<()> {
        open::that(env!("CARGO_PKG_HOMEPAGE")).map_err(Into::into)
    }

    pub fn open_repo(&self) -> Result<()> {
        open::that(env!("CARGO_PKG_REPOSITORY")).map_err(Into::into)
    }
}

pub struct UiBuffers {
    pub action: Option<UiAction>,
    pub config: Config,
    pub games_filter: String,
    pub hbc_apps_filter: String,
    pub osc_apps_filter: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub choose_mount_point: FileDialog,
    pub choose_games: FileDialog,
    pub choose_hbc_apps: FileDialog,
    pub choose_file_to_push: FileDialog,
    pub choose_archive_path: FileDialog,
    pub archiving_game_i: u16,
    pub notifications: Notifications,
}

impl UiBuffers {
    pub fn new(config: Config) -> Self {
        let choose_mount_point = FileDialog::new().as_modal(true);

        let choose_games = FileDialog::new()
            .as_modal(true)
            .add_file_filter_extensions(
                "Nintendo Optical Disc",
                extensions::SUPPORTED_INPUT_EXTENSIONS.to_vec(),
            )
            .default_file_filter("Nintendo Optical Disc");

        let choose_hbc_apps = FileDialog::new()
            .as_modal(true)
            .add_file_filter_extensions("HBC App (zip)", vec!["zip", "ZIP"])
            .default_file_filter("HBC App (zip)");

        let choose_file_to_push = FileDialog::new()
            .as_modal(true)
            .add_file_filter_extensions("HBC App (zip/dol/elf)", vec!["zip", "dol", "elf"])
            .default_file_filter("HBC App (zip/dol/elf)");

        let choose_archive_path = FileDialog::new()
            .as_modal(true)
            .add_save_extension(ArchiveFormat::Rvz.as_str(), ArchiveFormat::Rvz.extension())
            .add_save_extension(ArchiveFormat::Iso.as_str(), ArchiveFormat::Iso.extension())
            .default_save_extension(config.contents.archive_format.as_str());

        Self {
            action: None,
            config,
            games_filter: String::new(),
            hbc_apps_filter: String::new(),
            osc_apps_filter: String::new(),
            show_wii: true,
            show_gc: true,
            choose_mount_point,
            choose_games,
            choose_hbc_apps,
            choose_file_to_push,
            choose_archive_path,
            archiving_game_i: u16::MAX, // will be set later, panics if it's not
            notifications: Notifications::new(),
        }
    }

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.write() {
            self.notifications.show_err(e);
        }
    }
}

impl AppWrapper {
    fn update_title(&self, ctx: &egui::Context) {
        let title = format!(
            "{} • {} • {} ({})",
            env!("CARGO_PKG_NAME"),
            self.state.current_view.title(),
            self.ui_buffers.config.get_drive_path_str(),
            util::get_disk_usage(&self.ui_buffers.config.contents.mount_point)
        );

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    fn forget_osc_icons(&self, ctx: &egui::Context) {
        for osc_app in &self.state.osc_apps {
            ctx.forget_image(&osc_app.icon_uri);
        }
    }

    pub fn is_mount_point_known(&self) -> bool {
        known_mount_points::check(
            &self.state.data_dir,
            &self.ui_buffers.config.contents.mount_point,
        )
        .unwrap_or(true)
    }

    pub fn update_filtered_games(&mut self) {
        self.state.filtered_wii_games_size = Size::from_bytes(0);
        self.state.filtered_gc_games_size = Size::from_bytes(0);

        self.state.filtered_games.clear();
        self.state.filtered_wii_games.clear();
        self.state.filtered_gc_games.clear();

        if self.ui_buffers.games_filter.is_empty() {
            self.state
                .filtered_games
                .extend(0..self.state.games.len() as u16);

            for (i, game) in self.state.games.iter().enumerate() {
                if game.is_wii {
                    self.state.filtered_wii_games.push(i as u16);
                    self.state.filtered_wii_games_size += game.size;
                } else {
                    self.state.filtered_gc_games.push(i as u16);
                    self.state.filtered_gc_games_size += game.size;
                }
            }
        } else {
            let game_search = self.ui_buffers.games_filter.to_lowercase();

            for (i, game) in self.state.games.iter().enumerate() {
                if game.search_str.contains(&game_search) {
                    let i = i as u16;

                    self.state.filtered_games.push(i);

                    if game.is_wii {
                        self.state.filtered_wii_games.push(i);
                        self.state.filtered_wii_games_size += game.size;
                    } else {
                        self.state.filtered_gc_games.push(i);
                        self.state.filtered_gc_games_size += game.size;
                    }
                }
            }
        };
    }

    pub fn update_filtered_hbc_apps(&mut self) {
        self.state.filtered_hbc_apps.clear();
        self.state.filtered_hbc_apps_size = Size::from_bytes(0);

        if self.ui_buffers.hbc_apps_filter.is_empty() {
            self.state
                .filtered_hbc_apps
                .extend(0..self.state.hbc_apps.len() as u16);

            for hbc_app in &self.state.hbc_apps {
                self.state.filtered_hbc_apps_size += hbc_app.size;
            }

            return;
        }

        let hbc_app_search = self.ui_buffers.hbc_apps_filter.to_lowercase();
        for (i, hbc_app) in self.state.hbc_apps.iter().enumerate() {
            if hbc_app.search_str.contains(&hbc_app_search) {
                self.state.filtered_hbc_apps.push(i as u16);
                self.state.filtered_hbc_apps_size += hbc_app.size;
            }
        }
    }

    pub fn update_filtered_osc_apps(&mut self) {
        self.state.filtered_osc_apps.clear();

        if self.ui_buffers.osc_apps_filter.is_empty() {
            self.state
                .filtered_osc_apps
                .extend(0..self.state.osc_apps.len() as u16);

            return;
        }

        let osc_app_search = self.ui_buffers.osc_apps_filter.to_lowercase();
        for (i, osc_app) in self.state.osc_apps.iter().enumerate() {
            if osc_app.search_str.contains(&osc_app_search) {
                self.state.filtered_osc_apps.push(i as u16);
            }
        }
    }

    pub fn refresh_games(&mut self) {
        self.state.games = games::list(
            &self.ui_buffers.config.contents.mount_point,
            &self.state.titles,
        );

        games::sort(
            &mut self.state.games,
            SortBy::None,
            self.ui_buffers.config.contents.sort_by,
        );

        self.update_filtered_games();

        // Make sure that all games have covers
        covers::spawn_download_covers_task(
            &self.state.task_processor,
            self.ui_buffers.config.contents.mount_point.clone(),
            self.state.games.clone().into_boxed_slice(),
        );
    }

    pub fn refresh_hbc_apps(&mut self) {
        self.state.hbc_apps = hbc_apps::list(&self.ui_buffers.config.contents.mount_point);

        hbc_apps::sort(
            &mut self.state.hbc_apps,
            SortBy::None,
            self.ui_buffers.config.contents.sort_by,
        );

        self.update_filtered_hbc_apps();
    }

    pub fn apply_sorting(&mut self) {
        games::sort(
            &mut self.state.games,
            self.state.prev_sort_by,
            self.ui_buffers.config.contents.sort_by,
        );

        hbc_apps::sort(
            &mut self.state.hbc_apps,
            self.state.prev_sort_by,
            self.ui_buffers.config.contents.sort_by,
        );

        self.update_filtered_games();
        self.update_filtered_hbc_apps();
    }

    // Process files selected in FileDialogs
    fn process_dialog_files(&mut self, ctx: &egui::Context) {
        if let Some(mut paths) = self.ui_buffers.choose_games.take_picked_multiple() {
            // We'll get those later with get_overflow_file
            paths.retain(|path| !path.ends_with(".part1.iso"));

            let discs = paths
                .into_iter()
                .filter_map(|path| DiscInfo::from_main_file(path).ok())
                .filter(|info| {
                    self.state
                        .games
                        .iter()
                        .all(|game| game.id != info.header.game_id)
                })
                .collect::<Box<[_]>>();

            if discs.is_empty() {
                self.ui_buffers
                    .notifications
                    .show_info("No new games were selected");
            } else {
                self.state.current_modal = Some(ui::Modal::ConvertGames(discs));
            }
        }

        if let Some(path) = self.ui_buffers.choose_mount_point.take_picked() {
            self.ui_buffers.config.contents.mount_point = path;

            if !self.is_mount_point_known() {
                self.ui_buffers.notifications.show_info_no_duration("New Drive detected, a path normalization run is recommended\nYou can find it in the 🔧 Tools page");
            }

            self.refresh_games();
            self.refresh_hbc_apps();

            self.update_title(ctx);
            self.ui_buffers.save_config();
        }

        if let Some(out_path) = self.ui_buffers.choose_archive_path.take_picked() {
            let i = self.ui_buffers.archiving_game_i;

            let game = &self.state.games[i as usize];

            match archive::spawn_archive_game_task(
                &self.state.task_processor,
                game.path.clone(),
                out_path,
            ) {
                Ok(format) => {
                    self.ui_buffers.config.contents.archive_format = format;
                    self.ui_buffers.save_config();
                }
                Err(e) => self.ui_buffers.notifications.show_err(e),
            }
        }

        if let Some(path) = self.ui_buffers.choose_file_to_push.take_picked() {
            let wii_ip = self.ui_buffers.config.contents.wii_ip.clone();
            wiiload::spawn_push_file_task(&self.state.task_processor, path, wii_ip.clone());

            self.ui_buffers.config.contents.wii_ip = wii_ip;
            self.ui_buffers.save_config();
        }

        if let Some(paths) = self.ui_buffers.choose_hbc_apps.take_picked_multiple() {
            hbc_apps::spawn_install_apps_task(
                &self.state.task_processor,
                &self.ui_buffers.config.contents,
                paths.into_boxed_slice(),
            );
        }
    }

    // Process current frame UI event (triggered by the user)
    fn process_ui_action(&mut self, ctx: &egui::Context) {
        if let Some(action) = self.ui_buffers.action.take() {
            match action {
                UiAction::OpenView(view) => {
                    if self.state.current_view == ui::View::Osc {
                        self.forget_osc_icons(ctx);
                    }

                    self.state.current_view = view;
                    self.update_title(ctx);
                }
                UiAction::OpenModal(modal) => {
                    self.state.current_modal = Some(modal);
                }
                UiAction::CloseModal => {
                    self.state.current_modal = None;
                }
                UiAction::RunNormalizePaths => {
                    if let Err(e) =
                        dir_layout::normalize_paths(&self.ui_buffers.config.contents.mount_point)
                    {
                        self.ui_buffers.notifications.show_err(e);
                    } else {
                        self.ui_buffers
                            .notifications
                            .show_success("Paths successfully normalized");
                    }
                }
                UiAction::RunDotClean => {
                    if let Err(e) =
                        util::run_dot_clean(&self.ui_buffers.config.contents.mount_point)
                    {
                        self.ui_buffers.notifications.show_err(e);
                    } else {
                        self.ui_buffers
                            .notifications
                            .show_success("dot_clean completed successfully");
                    }
                }
                UiAction::ApplyFilterGames => {
                    self.update_filtered_games();
                }
                UiAction::ApplyFilterHbcApps => {
                    self.update_filtered_hbc_apps();
                }
                UiAction::ApplyFilterOscApps => {
                    self.update_filtered_osc_apps();
                }
                UiAction::TriggerDownloadOscIcons => {
                    self.state.download_osc_icons();
                }
                UiAction::DeleteGame(i) => {
                    let game = &self.state.games[i as usize];

                    if let Err(e) = fs::remove_dir_all(&game.path) {
                        self.ui_buffers.notifications.show_err(e.into());
                    } else {
                        self.state.games.remove(i as usize);
                        self.update_filtered_games();
                        self.update_title(ctx);
                    }

                    self.state.current_modal = None;
                }
                UiAction::TriggerRefreshGames => {
                    self.refresh_games();
                    self.update_title(ctx);
                }
                UiAction::ApplySorting => {
                    self.apply_sorting();
                    self.state.prev_sort_by = self.ui_buffers.config.contents.sort_by;
                    self.ui_buffers.save_config();
                }
                UiAction::DeleteHbcApp(i) => {
                    let hbc_app = &self.state.hbc_apps[i as usize];

                    if let Err(e) = fs::remove_dir_all(&hbc_app.path) {
                        self.ui_buffers.notifications.show_err(e.into());
                    } else {
                        self.state.hbc_apps.remove(i as usize);
                        self.update_filtered_hbc_apps();
                        self.update_title(ctx);
                    }

                    self.state.current_modal = None;
                }
                UiAction::TriggerRefreshHbcApps => {
                    self.refresh_hbc_apps();
                    self.update_title(ctx);
                }
            }
        }
    }

    // Process background messages from task_processor
    fn process_background_messages(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.state.task_processor.msg_receiver.try_recv() {
            match msg {
                BackgroundMessage::NotifyInfo(i) => {
                    self.ui_buffers.notifications.show_info(&i);
                }
                BackgroundMessage::NotifyError(e) => {
                    self.ui_buffers.notifications.show_err(e);
                }
                BackgroundMessage::NotifySuccess(s) => {
                    self.ui_buffers.notifications.show_success(&s);
                }
                BackgroundMessage::UpdateStatus(string) => {
                    self.state.status = string;
                }
                BackgroundMessage::ClearStatus => {
                    self.state.status.clear();
                }
                BackgroundMessage::TriggerRefreshImage(uri) => {
                    ctx.forget_image(&uri);
                }
                BackgroundMessage::TriggerRefreshGames => {
                    self.refresh_games();
                    self.update_title(ctx);
                }
                BackgroundMessage::TriggerRefreshHbcApps => {
                    self.refresh_hbc_apps();
                    self.update_title(ctx);
                }
                BackgroundMessage::GotNewVersion(version) => {
                    let info = UpdateInfo::from_version(version);
                    self.ui_buffers
                        .notifications
                        .show_info_no_duration(&info.ui_text);
                    self.state.update_info = Some(info);
                }
                BackgroundMessage::GotTitles(titles) => {
                    self.state.titles = Some(titles);
                    self.refresh_games();
                    self.update_title(ctx);
                }
                BackgroundMessage::GotOscApps(osc_apps) => {
                    self.state.osc_apps = osc_apps;
                    self.update_filtered_osc_apps();
                    self.state.check_for_hbc_app_updates();
                }
                BackgroundMessage::GotWiitdb(data) => {
                    self.state.wiitdb = Some(data);
                }
            }

            ctx.request_repaint();
        }
    }

    // Process OSC icon updates
    fn process_osc_messages(&mut self, ctx: &egui::Context) {
        if let Some(receiver) = self.state.downloading_osc_icons.as_mut() {
            while let Ok(icon_uri) = receiver.try_recv() {
                ctx.forget_image(&icon_uri);
            }
        }
    }
}

impl eframe::App for AppWrapper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::root::update(ctx, &self.state, &mut self.ui_buffers);

        self.process_dialog_files(ctx);
        self.process_ui_action(ctx);
        self.process_background_messages(ctx);
        self.process_osc_messages(ctx);
    }
}
