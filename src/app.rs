// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::{Message, process_msg};
use crate::ui::accent::AccentColor;
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
    tasks::TaskProcessor,
    titles::Titles,
    ui::{self, Modal},
    updater::UpdateInfo,
    util, wiiload,
    wiitdb::{self, GameInfo},
};
use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use eframe::egui::OpenUrl;
use egui_file_dialog::FileDialog;
use size::Size;
use smallvec::SmallVec;
use std::{fs, path::PathBuf, thread};

pub struct App {
    pub msg_sender: Sender<Message>,
    pub msg_receiver: Receiver<Message>,
    pub data_dir: PathBuf,
    pub task_processor: TaskProcessor,
    pub titles: Titles,
    pub has_osc_icons_downlading_started: bool,
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

impl App {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        let (msg_sender, msg_receiver) = unbounded();
        let task_processor = TaskProcessor::new(msg_sender.clone());

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
            msg_sender,
            msg_receiver,
            data_dir,
            current_view: ui::View::Games,
            update_info: None,
            games: Vec::new(),
            filtered_games: SmallVec::new(),
            filtered_wii_games: SmallVec::new(),
            filtered_gc_games: SmallVec::new(),
            filtered_wii_games_size: Size::from_bytes(0),
            filtered_gc_games_size: Size::from_bytes(0),
            titles: Titles::empty(),
            task_processor,
            has_osc_icons_downlading_started: false,
            hbc_apps: Vec::new(),
            filtered_hbc_apps: SmallVec::new(),
            filtered_hbc_apps_size: Size::from_bytes(0),
            osc_apps: Box::new([]),
            filtered_osc_apps: SmallVec::new(),
            status: String::new(),
            wiitdb: None,
            current_modal: None,
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

    pub fn send_msg(&self, msg: Message) {
        self.msg_sender.send(msg).expect("Failed to send message");
    }

    pub fn open_view(&mut self, ctx: &egui::Context, view: ui::View) {
        if self.current_view == ui::View::Osc {
            self.forget_osc_icons(ctx);
        }

        self.current_view = view;
        self.update_title(ctx);
    }

    pub fn close_modal(&mut self) {
        self.current_modal = None;
    }

    pub fn download_osc_icons(&mut self) {
        let icons_dir = self.data_dir.join("osc-icons");
        let msg_sender = self.msg_sender.clone();
        let osc_apps = self.osc_apps.clone();

        thread::spawn(move || {
            for osc_app in osc_apps {
                if osc::download_icon(&osc_app.meta, &icons_dir).is_ok() {
                    let _ = msg_sender.send(Message::TriggerRefreshImage(osc_app.icon_uri));
                }
            }
        });

        self.has_osc_icons_downlading_started = true;
    }

    pub fn get_game_info(&self, game_id: [u8; 6]) -> Option<GameInfo> {
        self.wiitdb
            .as_ref()
            .and_then(|db| db.lookup(game_id))
            .cloned()
    }

    pub fn open_data_dir(&mut self) {
        if let Err(e) = open::that(&self.data_dir) {
            self.notifications.show_err(e.into());
        }
    }

    pub fn open_game_dir(&mut self, game_i: u16) {
        let game = &self.games[game_i as usize];
        if let Err(e) = open::that(&game.path) {
            self.notifications.show_err(e.into());
        }
    }

    pub fn open_hbc_app_dir(&mut self, hbc_app_i: u16) {
        let hbc_app = &self.hbc_apps[hbc_app_i as usize];
        if let Err(e) = open::that(&hbc_app.path) {
            self.notifications.show_err(e.into());
        }
    }

    pub fn open_osc_app_info(&mut self, ctx: &egui::Context, osc_app_i: u16) {
        let osc_app = &self.osc_apps[osc_app_i as usize];
        let url = &osc_app.info_url;
        ctx.open_url(OpenUrl::new_tab(url));
    }

    pub fn open_update_info_url(&mut self, ctx: &egui::Context) {
        if let Some(update_info) = &self.update_info {
            let url = &update_info.url;
            ctx.open_url(OpenUrl::new_tab(url));
        }
    }

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.write() {
            self.notifications.show_err(e);
        }
    }

    pub fn set_accent_color(&mut self, ctx: &egui::Context, accent_color: AccentColor) {
        ctx.all_styles_mut(|style| {
            style.visuals.selection.bg_fill = accent_color.into();
        });

        self.config.contents.accent_color = accent_color;
        self.save_config();
    }

    pub fn run_dot_clean(&mut self) {
        if let Err(e) = util::run_dot_clean(&self.config.contents.mount_point) {
            self.notifications.show_err(e);
        } else {
            self.notifications
                .show_success("dot_clean completed successfully");
        }
    }

    pub fn run_normalize_paths(&mut self) {
        if let Err(e) = dir_layout::normalize_paths(&self.config.contents.mount_point) {
            self.notifications.show_err(e);
        } else {
            self.notifications
                .show_success("Paths successfully normalized");
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

    pub fn forget_osc_icons(&self, ctx: &egui::Context) {
        for osc_app in &self.osc_apps {
            ctx.forget_image(&osc_app.icon_uri);
        }
    }

    pub fn update_filtered_games(&mut self) {
        self.filtered_wii_games_size = Size::from_bytes(0);
        self.filtered_gc_games_size = Size::from_bytes(0);

        self.filtered_games.clear();
        self.filtered_wii_games.clear();
        self.filtered_gc_games.clear();

        if self.games_filter.is_empty() {
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
            let game_search = self.games_filter.to_lowercase();

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

        if self.hbc_apps_filter.is_empty() {
            self.filtered_hbc_apps.extend(0..self.hbc_apps.len() as u16);

            for hbc_app in &self.hbc_apps {
                self.filtered_hbc_apps_size += hbc_app.size;
            }

            return;
        }

        let hbc_app_search = self.hbc_apps_filter.to_lowercase();
        for (i, hbc_app) in self.hbc_apps.iter().enumerate() {
            if hbc_app.search_str.contains(&hbc_app_search) {
                self.filtered_hbc_apps.push(i as u16);
                self.filtered_hbc_apps_size += hbc_app.size;
            }
        }
    }

    pub fn update_filtered_osc_apps(&mut self) {
        self.filtered_osc_apps.clear();

        if self.osc_apps_filter.is_empty() {
            self.filtered_osc_apps.extend(0..self.osc_apps.len() as u16);
            return;
        }

        let osc_app_search = self.osc_apps_filter.to_lowercase();
        for (i, osc_app) in self.osc_apps.iter().enumerate() {
            if osc_app.search_str.contains(&osc_app_search) {
                self.filtered_osc_apps.push(i as u16);
            }
        }
    }

    pub fn refresh_games(&mut self) {
        self.games = games::list(&self.config.contents.mount_point, &self.titles);

        let is_known = known_mount_points::check(self).unwrap_or(true);

        if !self.games.is_empty() {
            if !is_known {
                self.notifications.show_info_no_duration("New Drive detected, a path normalization run is recommended\nYou can find it in the 🔧 Tools page");
            }

            games::sort(&mut self.games, SortBy::None, self.config.contents.sort_by);
            covers::spawn_download_covers_task(self);
        }

        self.update_filtered_games();
    }

    pub fn refresh_hbc_apps(&mut self) {
        self.hbc_apps = hbc_apps::list(&self.config.contents.mount_point, &self.osc_apps);

        hbc_apps::sort(
            &mut self.hbc_apps,
            SortBy::None,
            self.config.contents.sort_by,
        );

        self.update_filtered_hbc_apps();
    }

    pub fn apply_sorting(&mut self, sort_by: SortBy) {
        games::sort(&mut self.games, self.config.contents.sort_by, sort_by);
        hbc_apps::sort(&mut self.hbc_apps, self.config.contents.sort_by, sort_by);

        self.update_filtered_games();
        self.update_filtered_hbc_apps();

        self.config.contents.sort_by = sort_by;
        self.save_config();
    }

    // Process files selected in FileDialogs
    fn process_dialog_files(&mut self, ctx: &egui::Context) {
        if let Some(mut paths) = self.choose_games.take_picked_multiple() {
            // We'll get those later with get_overflow_file
            paths.retain(|path| !path.ends_with(".part1.iso"));

            let discs = paths
                .into_iter()
                .filter_map(|path| DiscInfo::from_main_file(path).ok())
                .filter(|info| self.games.iter().all(|game| game.id != info.header.game_id))
                .collect::<Box<[_]>>();

            if discs.is_empty() {
                self.notifications.show_info("No new games were selected");
            } else {
                self.current_modal = Some(ui::Modal::ConvertGames(discs));
            }
        }

        if let Some(path) = self.choose_mount_point.take_picked() {
            self.config.contents.mount_point = path;

            self.refresh_games();
            self.refresh_hbc_apps();

            self.update_title(ctx);
            self.save_config();
        }

        if let Some(out_path) = self.choose_archive_path.take_picked() {
            let i = self.archiving_game_i;

            let game = &self.games[i as usize];

            match archive::spawn_archive_game_task(self, game.path.clone(), out_path) {
                Ok(format) => {
                    self.config.contents.archive_format = format;
                    self.save_config();
                }
                Err(e) => self.notifications.show_err(e),
            }
        }

        if let Some(path) = self.choose_file_to_push.take_picked() {
            wiiload::spawn_push_file_task(self, path);
            self.save_config();
        }

        if let Some(paths) = self.choose_hbc_apps.take_picked_multiple() {
            hbc_apps::spawn_install_apps_task(self, paths.into_boxed_slice());
        }
    }

    pub fn delete_game(&mut self, ctx: &egui::Context, game_i: u16) {
        let game = &self.games[game_i as usize];

        if let Err(e) = fs::remove_dir_all(&game.path) {
            self.notifications.show_err(e.into());
        } else {
            self.games.remove(game_i as usize);
            self.update_filtered_games();
            self.update_title(ctx);
        }
    }

    pub fn delete_hbc_app(&mut self, ctx: &egui::Context, hbc_app_i: u16) {
        let hbc_app = &self.hbc_apps[hbc_app_i as usize];

        if let Err(e) = fs::remove_dir_all(&hbc_app.path) {
            self.notifications.show_err(e.into());
        } else {
            self.hbc_apps.remove(hbc_app_i as usize);
            self.update_filtered_hbc_apps();
            self.update_title(ctx);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ui::root::update(ctx, self);

        self.process_dialog_files(ctx);

        // Process messages from ui and background tasks
        while let Ok(msg) = self.msg_receiver.try_recv() {
            process_msg(self, ctx, msg);
            ctx.request_repaint();
        }
    }
}
