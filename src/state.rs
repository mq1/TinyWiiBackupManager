// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, ThemePreference},
    games::{
        conversion_state::ConversionState, covers::download_ui_covers, game::Game,
        games_state::GamesState, import::import_game,
    },
    homebrew::{self, homebrew_state::HomebrewState},
    messages::Message,
    notifications::{notification::Notification, notification_list::NotificationList},
    osc::osc_state::OscState,
    toolbox::{ToolContext, ToolboxItem},
    ui::{modals::Modal, pages::Page, theme},
    util::{data_dir::get_data_dir, drive_state::DriveState},
};
use anyhow::Context;
use iced::{
    Subscription, Task, Theme,
    time::{self, milliseconds},
};
use rfd::AsyncFileDialog;
use smol::{
    fs::{self, File},
    net::TcpStream,
};
use smol_str::{SmolStr, ToSmolStr, format_smolstr};
use std::{ffi::OsStr, path::PathBuf};
use wii_disc_info::game_id::GameID;
use wiiload::WIILOAD_PORT;

#[derive(Default)]
pub(crate) struct AppState {
    pub data_dir: PathBuf,
    pub config: Config,
    pub notifications: NotificationList,
    pub drive: DriveState,
    pub games: GamesState,
    pub homebrew: HomebrewState,
    pub current_page: Page,
    pub current_modal: Option<Modal>,
    pub importing: ConversionState,
    pub exporting: ConversionState,
    pub hashing: ConversionState,
    pub import_queue: Vec<PathBuf>,
    pub osc_contents: OscState,
    pub animation_state: bool,
}

impl AppState {
    pub fn boot() -> (Self, Task<Message>) {
        let data_dir = get_data_dir().expect("Unable to get data directory");

        let state = Self {
            notifications: NotificationList::new(&data_dir),
            data_dir,
            ..Default::default()
        };

        let task = state.load_config_task();

        (state, task)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(500)).map(|_| Message::ToggleAnimationState)
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.theme_preference {
            ThemePreference::System => theme::system(),
            ThemePreference::Light => Some(theme::light()),
            ThemePreference::Dark => Some(theme::dark()),
        }
    }

    pub fn write_config_task(&self) -> Task<Message> {
        let config = self.config.clone();
        Task::perform(async move { config.write().await }, |res| match res {
            Ok(()) => Message::NoOp,
            Err(e) => Message::Notify(e.into()),
        })
    }

    pub fn load_config_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();
        Task::perform(Config::load(data_dir), Message::GotConfig)
    }

    pub fn init_file_dialog_task(&self) -> Task<AsyncFileDialog> {
        iced::window::oldest()
            .and_then(|id| iced::window::run(id, |w| AsyncFileDialog::new().set_parent(w)))
    }

    pub fn get_games_task(&mut self) -> Task<Message> {
        self.games = GamesState::Loading;
        let mount_point = self.config.mount_point.clone();
        Task::perform(GamesState::load(mount_point), Message::GotGames)
    }

    pub fn download_ui_covers_task(&mut self) -> Task<Message> {
        if let GamesState::Loaded(games) = &self.games {
            let ids = games.get_all_game_ids().collect::<Box<[_]>>();
            let data_dir = self.data_dir.clone();
            let preferred_language = self.config.preferred_language;

            Task::stream(download_ui_covers(ids, data_dir, preferred_language))
                .map(Message::ReloadCover)
        } else {
            Task::none()
        }
    }

    pub fn get_homebrew_apps_task(&mut self) -> Task<Message> {
        self.homebrew = HomebrewState::Loading;
        let mount_point = self.config.mount_point.clone();
        Task::perform(HomebrewState::load(mount_point), Message::GotHomebrew)
    }

    pub fn get_drive_info_task(&mut self) -> Task<Message> {
        let mount_point = &self.config.mount_point;
        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        self.drive = DriveState::Loading;
        let mount_point = mount_point.clone();
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
                Err(e) => Message::CouldNotGetDiscInfo(e.to_smolstr()),
            },
        )
    }

    pub fn delete_dir_task(&mut self, path: PathBuf) -> Task<Message> {
        self.current_modal = None;

        Task::perform(fs::remove_dir_all(path), |res| {
            Message::DirDeleted(res.map_err(|e| e.to_smolstr()))
        })
    }

    pub fn import_homebrew_apps_task(&self, paths: Vec<PathBuf>) -> Task<Message> {
        let mount_point = self.config.mount_point.clone();

        Task::perform(
            homebrew::import(mount_point, paths, self.config.remove_sources_apps),
            |res| match res {
                Ok(n) => Message::HomebrewAppsImported(n),
                Err(e) => Message::Notify(e.into()),
            },
        )
    }

    pub fn import_games_task(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> Task<Message> {
        self.import_queue.extend(paths);

        if let ConversionState::Progress(_) = self.importing {
            return Task::none();
        }

        let task = if let Some(path) = self.import_queue.pop() {
            self.importing = ConversionState::Progress(SmolStr::default());
            Task::stream(import_game(path, self.config.clone(), self.drive.clone()))
                .map(Message::SetImporting)
        } else {
            self.notifications
                .push(Notification::info("Import queue is empty"));
            Task::none()
        };

        Task::batch([task, self.get_games_task(), self.get_drive_info_task()])
    }

    pub fn reload_cover(&mut self, game_id: GameID) {
        if let GamesState::Loaded(games) = &mut self.games {
            games.reload_cover(game_id, &self.data_dir);
        }
    }

    pub fn reload_all_covers(&mut self) {
        if let GamesState::Loaded(games) = &mut self.games {
            games.reload_all_covers(&self.data_dir);
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
        let wii_ip = self.config.wii_ip.to_smolstr();

        Task::perform(
            async move {
                let mut conn = TcpStream::connect((wii_ip.as_str(), WIILOAD_PORT)).await?;
                let filename = path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .context("invalid filename")?;
                let mut file = File::open(&path).await?;

                wiiload::compress_then_send_async(&mut conn, filename, &mut file).await?;

                let msg = format_smolstr!("Sent {filename} to {wii_ip}");
                Ok::<_, anyhow::Error>(msg)
            },
            |res| Message::Notify(res.into()),
        )
    }

    pub fn load_osc_contents_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();

        Task::perform(
            async move { OscState::load(&data_dir).await },
            Message::GotOscContents,
        )
    }
}
