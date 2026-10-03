// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, ThemePreference},
    games::{
        ImportEntry, covers::download_ui_covers, game::Game, games_state::GamesState,
        import::import_game,
    },
    homebrew::{self, homebrew_state::HomebrewState},
    long_operation::{LongOperationKind, LongOperationState, LongOperations},
    message::Message,
    notifications::{notification::Notification, notification_list::NotificationList},
    osc::{self, osc_state::OscState},
    toolbox::{ToolContext, ToolboxItem},
    ui::{modals::Modal, pages::Page, theme},
    util::{self, drive_state::DriveState, updates},
};
use anyhow::{Context, Result};

use iced::{
    Subscription, Task, Theme,
    time::{self, milliseconds},
};
use nod::{
    common::PartitionKind,
    read::{DiscOptions, DiscReader},
};
use rfd::AsyncFileDialog;
use semver::Version;
use smol::{
    fs::{self, File},
    io::AsyncSeekExt,
    net::TcpStream,
};
use std::{
    ffi::OsStr,
    io::SeekFrom,
    path::{Path, PathBuf},
};
use wii_disc_info::game_id::GameID;
use wiiload::WIILOAD_PORT;

#[cfg(not(feature = "compress-fonts"))]
fn load_lucide() -> Task<Message> {
    iced::font::load(lucide_icons::LUCIDE_FONT_BYTES).map(|res| match res {
        Ok(()) => Message::NoOp,
        Err(e) => Message::Notify(Notification::error(format!(
            "Failed to load lucide icons: {e:?}"
        ))),
    })
}

#[cfg(feature = "compress-fonts")]
fn load_lucide() -> Task<Message> {
    Task::future(async {
        let compressed = include_bytes!(concat!(env!("OUT_DIR"), "/lucide-compressed.bin"));
        miniz_oxide::inflate::decompress_to_vec(compressed).unwrap()
    })
    .then(|decompressed| {
        iced::font::load(decompressed).map(|res| match res {
            Ok(()) => Message::NoOp,
            Err(e) => Message::Notify(Notification::error(format!(
                "Failed to load lucide icons: {e:?}"
            ))),
        })
    })
}

pub struct AppState {
    pub data_dir: &'static Path,
    pub config: Config,
    pub notifications: NotificationList,
    pub drive: DriveState,
    pub games: GamesState,
    pub homebrew: HomebrewState,
    pub current_page: Page,
    pub current_modal: Option<Modal>,
    pub long_operations: LongOperations,
    pub import_queue: Vec<ImportEntry>,
    pub osc_contents: OscState,
    pub animation_state: bool,
    pub new_version: Option<Version>,
}

impl AppState {
    pub fn boot(data_dir: &'static Path) -> impl Fn() -> (Self, Task<Message>) {
        move || {
            (
                Self {
                    data_dir,
                    config: Config::default(),
                    notifications: NotificationList::new(data_dir),
                    drive: DriveState::NotLoaded,
                    games: GamesState::NotLoaded,
                    homebrew: HomebrewState::NotLoaded,
                    current_page: Page::Games,
                    current_modal: None,
                    import_queue: Vec::new(),
                    osc_contents: OscState::NotLoaded,
                    animation_state: false,
                    new_version: None,
                    long_operations: LongOperations::new(),
                },
                Task::batch([
                    Task::perform(Config::load(data_dir), Message::GotConfig),
                    Task::perform(OscState::load(data_dir, false), Message::GotOscContents),
                    Task::perform(updates::check(), |res| match res {
                        Ok(Some(version)) => Message::GotUpdate(version),
                        Ok(None) => Message::NoOp,
                        Err(e) => Message::Notify(e.into()),
                    }),
                    load_lucide(),
                ]),
            )
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::event::listen_with(|event, _status, _id| {
                if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
                    Some(Message::FileDropped(path))
                } else {
                    None
                }
            }),
            time::every(milliseconds(500)).map(|_| Message::ToggleAnimationState),
        ])
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.theme_preference() {
            ThemePreference::System => theme::system(),
            ThemePreference::Light => Some(theme::light()),
            ThemePreference::Dark => Some(theme::dark()),
        }
    }

    pub fn init_file_dialog_task(&self) -> Task<AsyncFileDialog> {
        iced::window::oldest()
            .and_then(|id| iced::window::run(id, |w| AsyncFileDialog::new().set_parent(w)))
    }

    pub fn get_games_task(&mut self) -> Task<Message> {
        self.games = GamesState::Loading;
        let mount_point = self.config.mount_point().to_path_buf();
        Task::perform(GamesState::load(mount_point), Message::GotGames)
    }

    pub fn download_ui_covers_task(&mut self) -> Task<Message> {
        if let GamesState::Loaded(games) = &self.games {
            let ids = games.get_all_game_ids().collect::<Box<[_]>>();
            let preferred_language = self.config.preferred_language();

            Task::stream(download_ui_covers(ids, self.data_dir, preferred_language))
                .map(Message::ReloadCover)
        } else {
            Task::none()
        }
    }

    pub fn download_osc_icons_task(&mut self) -> Task<Message> {
        if let OscState::Loaded(apps) = &self.osc_contents {
            Task::stream(osc::icons::download_all_icons(apps.clone(), self.data_dir))
                .map(Message::ReloadOscIcon)
        } else {
            Task::none()
        }
    }

    pub fn get_homebrew_apps_task(&mut self) -> Task<Message> {
        self.homebrew = HomebrewState::Loading;
        let mount_point = self.config.mount_point().to_path_buf();
        Task::perform(HomebrewState::load(mount_point), Message::GotHomebrew)
    }

    pub fn get_drive_info_task(&mut self) -> Task<Message> {
        let mount_point = self.config.mount_point();
        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        self.drive = DriveState::Loading;
        let mount_point = mount_point.to_path_buf();
        Task::perform(DriveState::load(mount_point), Message::GotDrive)
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

    pub fn add_to_import_queue(&mut self, mut paths: Vec<ImportEntry>) {
        self.import_queue.append(&mut paths);
    }

    pub fn reload_cover(&mut self, game_id: GameID) {
        if let GamesState::Loaded(games) = &mut self.games {
            games.reload_cover(game_id, self.data_dir);
        }
    }

    pub fn reload_all_covers(&mut self) {
        if let GamesState::Loaded(games) = &mut self.games {
            games.reload_all_covers(self.data_dir);
        }
    }

    pub fn reload_osc_icon(&mut self, idx: usize) {
        if let OscState::Loaded(apps) = &mut self.osc_contents {
            apps.reload_icon(idx, self.data_dir);
        }
    }

    pub fn reload_all_osc_icons(&mut self) {
        if let OscState::Loaded(apps) = &mut self.osc_contents {
            apps.reload_all_icons(self.data_dir);
        }
    }

    pub fn run_tool(&mut self, tool: &'static ToolboxItem) -> Task<Message> {
        let game_ids = match &self.games {
            GamesState::Loaded(games) => games.get_all_game_ids().collect::<Box<[_]>>(),
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

    pub fn send_via_wiiload(&self, path: PathBuf) -> Task<Message> {
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
                    let mut file = File::open(&path).await?;
                    wiiload::compress_then_send_async(&mut conn, &filename, &mut file).await?;

                    format!("Sent {filename} to {wii_ip}")
                };

                Ok::<_, anyhow::Error>(msg)
            },
            |res| Message::Notify(res.into()),
        )
    }

    pub fn set_homebrew_apps(&mut self, homebrew: HomebrewState) {
        self.homebrew = homebrew;

        if let HomebrewState::Loaded(apps) = &mut self.homebrew
            && let OscState::Loaded(osc_apps) = &self.osc_contents
        {
            apps.set_osc_apps(osc_apps);
        }
    }

    pub fn set_osc_contents(&mut self, contents: OscState) {
        self.osc_contents = contents;

        if let OscState::Loaded(osc_apps) = &self.osc_contents
            && let HomebrewState::Loaded(apps) = &mut self.homebrew
        {
            apps.set_osc_apps(osc_apps);
        }
    }

    pub fn refresh_games_and_apps(&mut self) -> Task<Message> {
        Task::batch([
            self.get_games_task(),
            self.get_homebrew_apps_task(),
            self.get_drive_info_task(),
        ])
    }
}
