// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, message::ConfigMessage},
    long_operation::{Status, message::LongOperationsMessage},
    message::Message,
    notifications::notification::Notification,
    osc,
    state::AppState,
    ui::{dialogs, pages::Page},
    util::updates,
};
use anyhow::{Context, Result};
use iced::Task;
use smol::{
    channel::{bounded, unbounded},
    fs::{self, File},
    io::BufReader,
    net::TcpStream,
    unblock,
};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use twbm_common::wiiload_util;
use twbm_driveinfo::DriveInfoState;
use twbm_games::{
    calc_sha1::calc_sha1, export::export_game, game::Game,
    has_update_partition::has_update_partition, import::import_game, manager::GameManager,
};
use twbm_homebrew::manager::HomebrewAppManager;
use twbm_osc::{app::OscApp, manager::OscAppManager};
use twbm_toolbox::{
    ToolContext, ToolboxItem, covers::download_ui_covers, txtcodes::download_cheats,
};
use wiiload::WIILOAD_PORT;

impl AppState {
    pub fn init_task(&self) -> Task<Message> {
        Task::batch([
            Task::perform(Config::load(self.data_dir), Message::GotConfig),
            Task::perform(
                unblock(|| OscAppManager::load(self.data_dir, false)),
                Message::GotOsc,
            ),
            Task::perform(updates::check(), |res| match res {
                Ok(Some(version)) => Message::GotUpdate(version),
                Ok(None) => Message::NoOp,
                Err(e) => Message::Notify(e.into()),
            }),
        ])
    }

    pub fn get_games_task(&mut self) -> Task<Message> {
        let Some(mount_point) = self.config.contents.mount_point.as_deref() else {
            return Task::none();
        };

        let mount_point = <Box<Path>>::from(mount_point);
        let data_dir = self.data_dir;
        self.game_manager = GameManager::Loading;

        Task::perform(
            smol::unblock(move || GameManager::load(&mount_point, data_dir)),
            Message::GotGames,
        )
    }

    pub fn download_ui_covers_task(&mut self) -> Task<Message> {
        if let GameManager::Loaded(games) = &self.game_manager {
            let data_dir = self.data_dir;
            let game_ids = games.get_all_game_ids();
            let preferred_language = self.config.contents.preferred_language;

            let (tx, rx) = unbounded();
            std::thread::spawn(move || {
                for game_id in download_ui_covers(game_ids, data_dir, preferred_language) {
                    let _ = tx.try_send(game_id);
                }
            });

            Task::stream(rx).map(Message::ReloadCover)
        } else {
            Task::none()
        }
    }

    pub fn download_osc_icons_task(&mut self) -> Task<Message> {
        if let OscAppManager::Loaded(apps) = &self.osc {
            Task::stream(osc::icons::download_all_icons(apps.clone(), self.data_dir))
                .map(Message::ReloadOscIcon)
        } else {
            Task::none()
        }
    }

    pub fn get_homebrew_apps_task(&mut self) -> Task<Message> {
        let Some(mount_point) = self.config.contents.mount_point.as_deref() else {
            return Task::none();
        };

        let mount_point = <Box<Path>>::from(mount_point);
        self.set_homebrew_apps(HomebrewAppManager::Loading);

        Task::perform(
            unblock(move || HomebrewAppManager::load(&mount_point)),
            Message::GotHomebrew,
        )
    }

    pub fn get_drive_info_task(&mut self) -> Task<Message> {
        let Some(mount_point) = self.config.contents.mount_point.as_deref() else {
            return Task::none();
        };

        let mount_point = <Box<Path>>::from(mount_point);
        self.drive = DriveInfoState::Loading;

        Task::perform(
            unblock(move || DriveInfoState::load(&mount_point)),
            Message::GotDrive,
        )
    }

    pub fn get_disc_info_task(&self, game: Game) -> Task<Message> {
        Task::perform(unblock(move || game.get_disc_info()), |res| match res {
            Ok(meta) => Message::GotDiscInfo(meta),
            Err(e) => Message::Notify(Notification::error(e.to_string())),
        })
    }

    pub fn check_if_has_update_partition_task(&mut self, game: Game) -> Task<Message> {
        Task::perform(
            smol::unblock(move || has_update_partition(&game)),
            |res: Result<bool>| match res {
                Ok(has_update_partition) => {
                    Message::GotUpdatePartitionCheckResult(has_update_partition)
                }
                Err(e) => Message::Notify(Notification::error(e.to_string())),
            },
        )
    }

    pub fn delete_dir_task(&mut self, path: PathBuf) -> Task<Message> {
        self.current_modal = None;

        Task::perform(fs::remove_dir_all(path), |res| {
            Message::DirDeleted(res.map_err(|e| e.to_string()))
        })
    }

    pub fn import_homebrew_apps_task(&self, paths: Vec<PathBuf>) -> Task<Message> {
        let Some(mount_point) = self.config.contents.mount_point.as_deref() else {
            return Task::none();
        };

        let mount_point = <Box<Path>>::from(mount_point);
        let remove_sources = self.config.contents.remove_sources_apps;

        Task::perform(
            unblock(move || twbm_homebrew::import(&mount_point, &paths, remove_sources)),
            |res| match res {
                Ok(n) => Message::HomebrewAppsImported(n),
                Err(e) => Message::Notify(e.into()),
            },
        )
    }

    pub fn trigger_import_task(&mut self) -> Task<Message> {
        if let Some(entry) = self.import_queue.pop() {
            self.long_operations.import =
                Status::Progress(format!("⤓  Importing {}", entry.meta().game_title()));

            let (tx, rx) = bounded(1);
            let config = self.config.contents.clone();
            let has_4gib_filesize_limit = self.drive.has_4gib_filesize_limit();

            let _ = std::thread::spawn(move || {
                let send_progress = |msg| _ = tx.try_send(Status::Progress(msg));
                let _ = tx.send_blocking(
                    match import_game(&entry, &config, has_4gib_filesize_limit, send_progress) {
                        Ok(msg) => Status::Finished(msg),
                        Err(e) => Status::Errored(e.to_string()),
                    },
                );
            });

            Task::stream(rx)
                .map(LongOperationsMessage::SetImport)
                .map(Message::LongOperations)
        } else {
            self.notifications
                .push(Notification::info("Import queue is empty"));
            Task::none()
        }
    }

    pub fn run_tool_task(&mut self, tool: &'static ToolboxItem) -> Task<Message> {
        let game_ids = match &self.game_manager {
            GameManager::Loaded(games) => games.get_all_game_ids(),
            _ => Box::new([]),
        };

        let ctx = ToolContext {
            config: self.config.contents.clone(),
            game_ids,
        };

        self.notifications.push(Notification::info(format!(
            "Running tool \"{}\"",
            tool.label()
        )));

        Task::perform(unblock(move || tool.run(ctx)), |res| {
            Message::Notify(res.into())
        })
    }

    pub fn send_via_wiiload_task(&self, path: PathBuf) -> Task<Message> {
        let wii_ip: Box<str> = self.config.contents.wii_ip.as_str().into();

        Task::perform(
            async move {
                let filename: Box<str> = path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .context("invalid filename")?
                    .into();

                let is_zip = path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

                let mut conn = TcpStream::connect((wii_ip.as_ref(), WIILOAD_PORT)).await?;

                let msg = if is_zip {
                    let (data, excluded_files) = smol::unblock(move || {
                        let mut file = std::fs::File::open(path)?;
                        wiiload_util::rebuild_zip(&mut file)
                    })
                    .await?;

                    wiiload::send_slice_async(&mut conn, &filename, &data[..]).await?;

                    format!(
                        "Sent {filename} to {wii_ip} ({} excluded files)",
                        excluded_files.len()
                    )
                } else {
                    let mut reader = BufReader::new(File::open(path).await?);
                    wiiload::compress_then_send_async(&mut conn, &filename, &mut reader).await?;

                    format!("Sent {filename} to {wii_ip}")
                };

                Ok::<_, anyhow::Error>(msg)
            },
            |res| Message::Notify(res.into()),
        )
    }

    pub fn refresh_games_and_apps_task(&mut self) -> Task<Message> {
        Task::batch([
            self.get_games_task(),
            self.get_homebrew_apps_task(),
            self.get_drive_info_task(),
        ])
    }

    pub fn navigate_to_task(&mut self, page: Page) -> Task<Message> {
        self.current_page = page;

        if page == Page::Osc && matches!(self.osc, OscAppManager::Loaded(_)) {
            self.reload_all_osc_icons();
            self.download_osc_icons_task()
        } else {
            Task::none()
        }
    }

    pub fn load_osc_state_task(&self) -> Task<Message> {
        Task::perform(
            unblock(|| OscAppManager::load(self.data_dir, true)),
            Message::GotOsc,
        )
    }

    pub fn calc_game_sha1_task(&mut self, game: Game) -> Task<Message> {
        self.long_operations.hash = Status::Progress("".into());

        let (tx, rx) = bounded(1);

        let _ = std::thread::spawn(move || {
            let send_progress = |msg| _ = tx.try_send(Status::Progress(msg));
            let _ = tx.send_blocking(match calc_sha1(&game, send_progress) {
                Ok(msg) => Status::Finished(msg),
                Err(e) => Status::Errored(e.to_string()),
            });
        });

        Task::stream(rx)
            .map(LongOperationsMessage::SetHash)
            .map(Message::LongOperations)
    }

    pub fn pick_games_recursively_task(&self) -> Task<Message> {
        if let GameManager::Loaded(games) = &self.game_manager {
            let existing_ids = games.get_all_game_ids();

            dialogs::init_file_dialog_task().then(move |base| {
                dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
            })
        } else {
            Task::none()
        }
    }

    pub fn pick_games_task(&self) -> Task<Message> {
        if let GameManager::Loaded(games) = &self.game_manager {
            let existing_ids = games.get_all_game_ids();

            dialogs::init_file_dialog_task()
                .then(move |base| dialogs::make_pick_games_dialog_task(base, existing_ids.clone()))
        } else {
            Task::none()
        }
    }

    pub fn download_txt_codes_task(&mut self, game: Game) -> Task<Message> {
        let game_id = game.id();
        let config = self.config.contents.clone();

        self.notifications.push(Notification::info(format!(
            "Downloading cheats for {}",
            game.title()
        )));

        Task::perform(
            unblock(move || download_cheats(game_id, &config)),
            move |res| match res {
                Ok(()) => Notification::success(format!("Downloaded cheats for {}", game.title())),
                Err(err) => Notification::error(err.to_string()),
            },
        )
        .map(Message::Notify)
    }

    pub fn send_osc_app_via_wiiload_task(&mut self, app: OscApp) -> Task<Message> {
        let app_name: Box<str> = app.name().into();
        let wii_ip: Box<str> = self.config.contents.wii_ip.as_str().into();

        self.notifications.push(Notification::info(format!(
            "Sending {app_name} via wiiload"
        )));

        Task::batch([
            self.config
                .update(ConfigMessage::Write)
                .map(Message::Config),
            Task::perform(
                unblock({
                    let wii_ip = wii_ip.clone();
                    move || app.wiiload(&wii_ip)
                }),
                move |res| match res {
                    Ok(excluded_files) if excluded_files.is_empty() => {
                        Notification::success(format!("Sent {app_name} to {wii_ip}"))
                    }
                    Ok(excluded_files) => Notification::info(format!(
                        "Sent {app_name} to {wii_ip} ({} excluded files)",
                        excluded_files.len()
                    )),
                    Err(err) => Notification::error(err.to_string()),
                },
            )
            .map(Message::Notify),
        ])
    }

    pub fn export_game_task(&mut self, game: Game, out_path: PathBuf) -> Task<Message> {
        self.long_operations.export = Status::Progress("".into());

        let (tx, rx) = bounded(1);

        let _ = std::thread::spawn(move || {
            let send_progress = |msg| _ = tx.try_send(Status::Progress(msg));
            let _ = tx.send_blocking(match export_game(&game, &out_path, send_progress) {
                Ok(msg) => Status::Finished(msg),
                Err(e) => Status::Errored(e.to_string()),
            });
        });

        Task::stream(rx)
            .map(LongOperationsMessage::SetExport)
            .map(Message::LongOperations)
    }

    pub fn install_osc_app_task(&mut self, app: OscApp) -> Task<Message> {
        let Some(mount_point) = self.config.contents.mount_point.as_deref() else {
            return Task::none();
        };

        self.notifications
            .push(Notification::info(format!("Installing {}", app.name())));

        let app_name = app.name().into();
        let mount_point = <Box<Path>>::from(mount_point);

        Task::perform(
            unblock(move || app.install(&mount_point)),
            move |res| match res {
                Ok(()) => Message::OscAppInstalled(Ok(app_name)),
                Err(e) => Message::OscAppInstalled(Err(e.to_string())),
            },
        )
    }
}
