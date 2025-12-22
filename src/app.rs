// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::SUPPORTED_INPUT_EXTENSIONS;
use crate::messages::{Message, process_msg};
use crate::{
    config::{Config, SortBy},
    convert, covers, dir_layout,
    disc_info::DiscInfo,
    games::{self, Game},
    hbc_apps::{self, HbcApp},
    known_mount_points,
    notifications::Notifications,
    osc::{self, OscApp},
    tasks::TaskProcessor,
    ui::{self, Modal},
    util, wiitdb,
};
use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use nod::common::Format;
use semver::Version;
use size::Size;
use smallvec::SmallVec;
use std::ffi::OsStr;
use std::{fs, path::PathBuf, thread};

pub struct App {
    pub msg_sender: Sender<Message>,
    pub msg_receiver: Receiver<Message>,
    pub data_dir: PathBuf,
    pub task_processor: TaskProcessor,
    pub has_osc_icons_downlading_started: bool,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub games: Box<[Game]>,
    pub osc_apps: Box<[OscApp]>,
    pub filtered_games: SmallVec<[u16; 512]>,
    pub filtered_wii_games: SmallVec<[u16; 256]>,
    pub filtered_wii_games_size: Size,
    pub filtered_gc_games: SmallVec<[u16; 256]>,
    pub filtered_gc_games_size: Size,
    pub filtered_osc_apps: SmallVec<[u16; 512]>,
    pub filtered_hbc_apps: SmallVec<[u16; 64]>,
    pub filtered_hbc_apps_size: Size,
    pub hbc_apps: Box<[HbcApp]>,
    pub current_view: ui::View,
    pub update: Option<Version>,
    pub status: String,
    pub current_modal: Option<Modal>,
    pub config: Config,
    pub games_filter: String,
    pub hbc_apps_filter: String,
    pub osc_apps_filter: String,
    pub show_wii: bool,
    pub show_gc: bool,
    pub notifications: Notifications,
    pub current_game_info: Option<u16>,
    pub current_disc_info: Option<DiscInfo>,
    pub nod_gui_input_path: String,
    pub nod_gui_output_path: String,
}

impl App {
    pub fn new(data_dir: PathBuf) -> Self {
        let config = Config::load(&data_dir);

        let (msg_sender, msg_receiver) = unbounded();
        let task_processor = TaskProcessor::new(msg_sender.clone());

        Self {
            msg_sender,
            msg_receiver,
            data_dir,
            current_view: ui::View::Games,
            update: None,
            games: Box::new([]),
            filtered_games: SmallVec::new(),
            filtered_wii_games: SmallVec::new(),
            filtered_gc_games: SmallVec::new(),
            filtered_wii_games_size: Size::from_bytes(0),
            filtered_gc_games_size: Size::from_bytes(0),
            task_processor,
            has_osc_icons_downlading_started: false,
            hbc_apps: Box::new([]),
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
            notifications: Notifications::new(),
            current_game_info: None,
            current_disc_info: None,
            nod_gui_input_path: String::new(),
            nod_gui_output_path: String::new(),
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

    pub fn save_config(&mut self) {
        if let Err(e) = self.config.write() {
            self.notifications.show_err(e);
        }
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

    pub fn refresh_games(&mut self, download_covers: bool) {
        self.games = games::list(self).into_boxed_slice();

        let is_known = known_mount_points::check(self).unwrap_or(true);

        if !self.games.is_empty() {
            if !is_known {
                self.notifications.show_info_no_duration("New Drive detected, a path normalization run is recommended\nYou can find it in the 🔧 Tools page");
            }

            games::sort(&mut self.games, SortBy::None, self.config.contents.sort_by);

            if download_covers {
                covers::spawn_cache_covers_task(self);
            }
        }

        self.update_filtered_games();
    }

    pub fn refresh_hbc_apps(&mut self) {
        self.hbc_apps =
            hbc_apps::list(&self.config.contents.mount_point, &self.osc_apps).into_boxed_slice();

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

    pub fn update_mount_point(&mut self, ctx: &egui::Context, path: PathBuf) {
        self.config.contents.mount_point = path;

        self.refresh_games(true);
        self.refresh_hbc_apps();
        wiitdb::spawn_load_wiitdb_task(self);

        self.update_title(ctx);
        self.save_config();
    }

    pub fn run_add_games(&mut self, frame: &eframe::Frame, mut paths: Vec<PathBuf>) {
        paths.retain(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| SUPPORTED_INPUT_EXTENSIONS.contains(&ext))
        });

        // We'll get those later with get_overflow_file
        paths.retain(|path| !path.ends_with(".part1.iso"));

        // Discard already present games and duplicates
        let existing_ids = self.games.iter().map(|game| game.id).collect::<Box<[_]>>();
        let discs = paths
            .into_iter()
            .filter_map(|path| {
                if path.extension() == Some(OsStr::new("zip")) {
                    DiscInfo::from_zip_file(&path).ok()
                } else {
                    DiscInfo::from_path(path).ok()
                }
            })
            .filter(|info| !existing_ids.contains(&info.id))
            .collect::<Box<[_]>>();

        if discs.is_empty() {
            self.notifications.show_info("No new games were selected");
        } else if ui::dialogs::confirm_add_games(frame, &discs) {
            convert::spawn_add_games_task(self, discs);
        }
    }

    pub fn add_games(&mut self, frame: &eframe::Frame) {
        let games = ui::dialogs::choose_games(frame);
        if !games.is_empty() {
            self.run_add_games(frame, games);
        }
    }

    pub fn add_games_from_dir(&mut self, frame: &eframe::Frame) {
        if let Some(src_dir) = ui::dialogs::choose_src_dir(frame) {
            let paths = util::scan_for_discs(&src_dir);
            self.run_add_games(frame, paths);
        }
    }

    pub fn archive_game(&mut self, i: u16, frame: &eframe::Frame) {
        let game = &self.games[i as usize];

        if let Ok(disc_info) = DiscInfo::from_game_dir(&game.path) {
            if let Some(dest_dir) = ui::dialogs::choose_dest_dir(frame) {
                let ext = if self.config.contents.archive_format == Format::Rvz {
                    "rvz"
                } else {
                    "iso"
                };

                let out_path = dest_dir.join(format!("{}.{}", disc_info.title, ext));

                convert::spawn_conv_game_task(self, disc_info.disc_path, out_path);
            }
        } else {
            self.notifications.show_err(anyhow!("Disc not found"));
        }
    }

    pub fn strip_game(&mut self, frame: &eframe::Frame) -> Result<()> {
        let disc_info = self
            .current_disc_info
            .clone()
            .ok_or(anyhow!("No disc selected"))?;

        if ui::dialogs::confirm_strip_game(frame, &disc_info.title) {
            convert::spawn_strip_game_task(self, disc_info);
        }

        Ok(())
    }

    pub fn delete_game(&self, frame: &eframe::Frame, game_i: u16) -> Result<()> {
        let game = &self.games[game_i as usize];

        if ui::dialogs::delete_game(frame, &game.display_title) {
            fs::remove_dir_all(&game.path)?;
            self.msg_sender.send(Message::TriggerRefreshGames(false))?;
        }

        Ok(())
    }

    pub fn delete_hbc_app(&self, frame: &eframe::Frame, hbc_app_i: u16) -> Result<()> {
        let hbc_app = &self.hbc_apps[hbc_app_i as usize];

        if ui::dialogs::delete_hbc_app(frame, &hbc_app.meta.name) {
            fs::remove_dir_all(&hbc_app.path)?;
            self.msg_sender.send(Message::TriggerRefreshHbcApps)?;
        }

        Ok(())
    }

    pub fn update_game_info(&mut self, game_i: u16) {
        let game = &self.games[game_i as usize];

        self.current_game_info = self.wiitdb.as_ref().and_then(|db| db.lookup(game.id));
        self.current_disc_info = DiscInfo::from_game_dir(&game.path).ok();
    }

    pub fn run_strip_all_games(&mut self, frame: &eframe::Frame) {
        if ui::dialogs::confirm_strip_all_games(frame)
            && !convert::spawn_strip_all_games_tasks(self)
        {
            self.notifications
                .show_info("No additional update partitions to remove were found");
        }
    }

    pub fn run_single_conversion(&mut self, frame: &eframe::Frame) {
        if ui::dialogs::confirm_single_conversion(
            frame,
            &self.nod_gui_input_path,
            &self.nod_gui_output_path,
        ) {
            let in_path = PathBuf::from(&self.nod_gui_input_path);
            let out_path = PathBuf::from(&self.nod_gui_output_path);

            convert::spawn_conv_game_task(self, in_path, out_path);
        }
    }

    pub fn cancel_tasks(&mut self, frame: &eframe::Frame) {
        if ui::dialogs::confirm_cancel_tasks(frame) {
            self.task_processor.cancel_pending();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ui::root::update(ctx, frame, self);

        // Process messages from ui and background tasks
        while let Ok(msg) = self.msg_receiver.try_recv() {
            process_msg(self, ctx, frame, msg);
            ctx.request_repaint();
        }
    }
}
