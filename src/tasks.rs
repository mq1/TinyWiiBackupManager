// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, message::ConfigMessage},
    games::{
        calc_sha1::calc_sha1, covers::download_ui_covers, export::export_game, game::Game,
        games_state::GameList, import::import_game, txtcodes::download_cheats,
    },
    homebrew::{self, homebrew_state::HomebrewAppList},
    long_operation::{LongOperationKind, LongOperationState},
    message::Message,
    notifications::notification::Notification,
    osc::{self, osc_app::OscApp, osc_state::OscAppList},
    state::{AppState, LazyState},
    toolbox::{ToolContext, ToolboxItem},
    ui::{dialogs, pages::Page},
    util::{self, drive_state::DriveInfo, updates},
};
use anyhow::{Context, Result};
use iced::Task;
use nod::{
    common::PartitionKind,
    read::{DiscOptions, DiscReader},
};
use smol::{
    fs::{self, File},
    io::{AsyncSeekExt, BufReader},
    net::TcpStream,
};
use std::{
    ffi::OsStr,
    io::SeekFrom,
    path::{Path, PathBuf},
};
use wiiload::WIILOAD_PORT;

impl AppState {
    pub fn init_task(&self) -> Task<Message> {
        Task::batch([
            Task::perform(Config::load(self.data_dir), Message::GotConfig),
            Task::perform(
                OscAppList::load(self.data_dir, false),
                Message::GotOscContents,
            ),
            Task::perform(updates::check(), |res| match res {
                Ok(Some(version)) => Message::GotUpdate(version),
                Ok(None) => Message::NoOp,
                Err(e) => Message::Notify(e.into()),
            }),
        ])
    }

    pub fn get_games_task(&mut self) -> Task<Message> {
        self.games = LazyState::Loading;
        let mount_point: Box<Path> = self.config.mount_point().into();

        Task::perform(
            async move { GameList::load(&mount_point).await },
            Message::GotGames,
        )
    }

    pub fn download_ui_covers_task(&mut self) -> Task<Message> {
        if let LazyState::Loaded(games) = &self.games {
            let ids = games.get_all_game_ids().collect::<Box<[_]>>();
            let preferred_language = self.config.preferred_language();

            Task::stream(download_ui_covers(ids, self.data_dir, preferred_language))
                .map(Message::ReloadCover)
        } else {
            Task::none()
        }
    }

    pub fn download_osc_icons_task(&mut self) -> Task<Message> {
        if let LazyState::Loaded(apps) = &self.osc_contents {
            Task::stream(osc::icons::download_all_icons(apps.clone(), self.data_dir))
                .map(Message::ReloadOscIcon)
        } else {
            Task::none()
        }
    }

    pub fn get_homebrew_apps_task(&mut self) -> Task<Message> {
        self.set_homebrew_apps(LazyState::Loading);
        let mount_point = self.config.mount_point().to_path_buf();
        Task::perform(HomebrewAppList::load(mount_point), Message::GotHomebrew)
    }

    pub fn get_drive_info_task(&mut self) -> Task<Message> {
        let mount_point = self.config.mount_point();
        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        self.drive = LazyState::Loading;
        let mount_point = mount_point.to_path_buf();
        Task::perform(DriveInfo::load(mount_point), Message::GotDrive)
    }

    pub fn get_disc_info_task(&self, game: Game) -> Task<Message> {
        Task::perform(
            async move {
                let disc_path = game.get_disc_path().await.context("disc not found")?;
                let mut file = File::open(&disc_path).await?;
                let meta = wii_disc_info::Meta::read_async(&mut file).await?;
                Ok::<_, anyhow::Error>(meta)
            },
            |res| match res {
                Ok(meta) => Message::GotDiscInfo(meta),
                Err(e) => Message::Notify(Notification::error(e.to_string())),
            },
        )
    }

    pub fn check_if_has_update_partition_task(&mut self, game: Game) -> Task<Message> {
        Task::perform(
            async move {
                smol::unblock(move || {
                    let disc_path = game.get_disc_path_blocking().context("disc not found")?;
                    let disc_reader = DiscReader::new(
                        disc_path,
                        &DiscOptions {
                            preloader_threads: 0,
                            ..Default::default()
                        },
                    )?;

                    let has_update_partition = disc_reader
                        .partitions()
                        .iter()
                        .any(|partition| partition.kind == PartitionKind::Update);

                    Ok(has_update_partition)
                })
                .await
            },
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
        let mount_point = self.config.mount_point().to_path_buf();
        let remove_sources = self.config.remove_sources_apps();

        Task::perform(
            homebrew::import(mount_point, paths, remove_sources),
            |res| match res {
                Ok(n) => Message::HomebrewAppsImported(n),
                Err(e) => Message::Notify(e.into()),
            },
        )
    }

    pub fn trigger_import_task(&mut self) -> Task<Message> {
        if let Some(entry) = self.import_queue.pop() {
            self.long_operations.set(
                LongOperationKind::Import,
                LongOperationState::Progress(format!("⤓  Importing {}", entry.meta().game_title())),
            );

            Task::stream(import_game(entry, self.config.clone(), self.drive.clone()))
                .map(|op_state| Message::SetLongOperationState(LongOperationKind::Import, op_state))
        } else {
            self.notifications
                .push(Notification::info("Import queue is empty"));
            Task::none()
        }
    }

    pub fn run_tool_task(&mut self, tool: &'static ToolboxItem) -> Task<Message> {
        let game_ids = match &self.games {
            LazyState::Loaded(games) => games.get_all_game_ids().collect::<Box<[_]>>(),
            _ => Box::new([]),
        };

        let ctx = ToolContext {
            config: self.config.clone(),
            game_ids,
        };

        self.notifications.push(Notification::info(format!(
            "Running tool \"{}\"",
            tool.label()
        )));

        Task::perform(tool.run(ctx), |res| Message::Notify(res.into()))
    }

    pub fn send_via_wiiload_task(&self, path: PathBuf) -> Task<Message> {
        let wii_ip: Box<str> = self.config.wii_ip().into();

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
                    let (file, excluded_files) = smol::unblock(move || {
                        let file = std::fs::File::open(path)?;
                        util::wiiload::rebuild_zip(file)
                    })
                    .await?;

                    let mut file = File::from(file);
                    file.seek(SeekFrom::Start(0)).await?;
                    let size = file.metadata().await?.len().try_into()?;
                    wiiload::send_async(&mut conn, &filename, &mut file, size).await?;

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

        if page == Page::Osc && matches!(self.osc_contents, LazyState::Loaded(_)) {
            self.reload_all_osc_icons();
            self.download_osc_icons_task()
        } else {
            Task::none()
        }
    }

    pub fn load_osc_state_task(&self) -> Task<Message> {
        Task::perform(
            OscAppList::load(self.data_dir, true),
            Message::GotOscContents,
        )
    }

    pub fn calc_game_sha1_task(&mut self, game: Game) -> Task<Message> {
        self.long_operations.set(
            LongOperationKind::Hash,
            LongOperationState::Progress(String::new()),
        );

        Task::stream(calc_sha1(game))
            .map(|op_state| Message::SetLongOperationState(LongOperationKind::Hash, op_state))
    }

    pub fn pick_games_recursively_task(&self) -> Task<Message> {
        if let LazyState::Loaded(games) = &self.games {
            let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

            dialogs::init_file_dialog_task().then(move |base| {
                dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
            })
        } else {
            Task::none()
        }
    }

    pub fn pick_games_task(&self) -> Task<Message> {
        if let LazyState::Loaded(games) = &self.games {
            let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

            dialogs::init_file_dialog_task()
                .then(move |base| dialogs::make_pick_games_dialog_task(base, existing_ids.clone()))
        } else {
            Task::none()
        }
    }

    pub fn download_txt_codes_task(&mut self, game: Game) -> Task<Message> {
        let game_title: Box<str> = game.title().into();
        let config = self.config.clone();

        self.notifications.push(Notification::info(format!(
            "Downloading cheats for {game_title}",
        )));

        Task::perform(
            async move { download_cheats(game.id(), &config).await },
            move |res| match res {
                Ok(()) => Notification::success(format!("Downloaded cheats for {game_title}")),
                Err(err) => Notification::error(err.to_string()),
            },
        )
        .map(Message::Notify)
    }

    pub fn send_osc_app_via_wiiload_task(&mut self, app: OscApp) -> Task<Message> {
        let app_name: Box<str> = app.name().into();
        let wii_ip: Box<str> = self.config.wii_ip().into();

        self.notifications.push(Notification::info(format!(
            "Sending {app_name} via wiiload"
        )));

        Task::batch([
            self.config
                .update(ConfigMessage::Write)
                .map(Message::Config),
            Task::perform(
                {
                    let wii_ip = wii_ip.clone();
                    async move { app.wiiload(&wii_ip).await }
                },
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

    pub fn set_long_operation_state_task(
        &mut self,
        kind: LongOperationKind,
        mut state: LongOperationState,
    ) -> Task<Message> {
        let should_continue = kind == LongOperationKind::Import
            && matches!(
                state,
                LongOperationState::Finished(_) | LongOperationState::Errored(_)
            );

        if let LongOperationState::Finished(success) = state {
            self.notifications.push(Notification::success(success));
            state = LongOperationState::Idle;
        } else if let LongOperationState::Errored(error) = state {
            self.notifications.push(Notification::error(error));
            state = LongOperationState::Idle;
        }

        self.long_operations.set(kind, state);

        if should_continue {
            Task::batch([
                self.trigger_import_task(),
                self.get_games_task(),
                self.get_drive_info_task(),
            ])
        } else {
            Task::none()
        }
    }

    pub fn export_game_task(&mut self, game: Game, out_path: PathBuf) -> Task<Message> {
        self.long_operations.set(
            LongOperationKind::Export,
            LongOperationState::Progress(String::new()),
        );

        Task::stream(export_game(game, out_path))
            .map(|op_state| Message::SetLongOperationState(LongOperationKind::Export, op_state))
    }

    pub fn install_osc_app_task(&mut self, app: OscApp) -> Task<Message> {
        self.notifications
            .push(Notification::info(format!("Installing {}", app.name())));

        let app_name = app.name().into();
        let mount_point: Box<Path> = self.config.mount_point().into();

        Task::perform(
            async move { app.install(&mount_point).await },
            move |res| match res {
                Ok(()) => Message::OscAppInstalled(Ok(app_name)),
                Err(e) => Message::OscAppInstalled(Err(e.to_string())),
            },
        )
    }
}
