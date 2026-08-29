// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    errors::Error,
    games::{covers::download_ui_covers, game::Game, game_list::GameList, import::import_game},
    homebrew::{self, homebrew_app_list::HomebrewAppList},
    messages::Message,
    notifications::{notification::Notification, notification_list::NotificationList},
    ui::{modals::Modal, pages::Page},
    util::{data_dir::get_data_dir, drive_info::DriveInfo},
};
use enumflags2::{BitFlags, bitflags};
use iced::{
    Subscription, Task,
    time::{self, milliseconds},
};
use rfd::AsyncFileDialog;
use smol::fs::{self, File};
use std::path::PathBuf;

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Ongoing {
    Converting,
    GettingGames,
    GettingHomebrewApps,
    GettingDriveInfo,
    DownloadingUiCovers,
    AnimationState,
}

#[derive(Default)]
pub(crate) struct AppState {
    pub(crate) data_dir: PathBuf,
    pub(crate) config: Config,
    pub(crate) notifications: NotificationList,
    pub(crate) drive_info: Option<DriveInfo>,
    pub(crate) games: GameList,
    pub(crate) homebrew_apps: HomebrewAppList,
    pub(crate) current_page: Page,
    pub(crate) current_modal: Option<Modal>,
    pub(crate) status: String,
    pub(crate) import_queue: Vec<PathBuf>,
    pub(crate) ongoing: BitFlags<Ongoing>,
}

impl AppState {
    pub fn boot() -> (Self, Task<Message>) {
        let data_dir = get_data_dir().expect("Unable to get data directory");

        let state = Self {
            data_dir,
            ..Default::default()
        };

        let task = state.load_config_task();

        (state, task)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(milliseconds(500)).map(|_| Message::ToggleAnimationState)
    }

    pub fn write_config_task(&self) -> Task<Message> {
        let config = self.config.clone();
        Task::perform(async move { config.write().await }, Message::WroteConfig)
    }

    pub fn load_config_task(&self) -> Task<Message> {
        let data_dir = self.data_dir.clone();
        Task::perform(Config::load(data_dir), Message::GotConfig)
    }

    pub fn init_file_dialog_task(&self) -> Task<AsyncFileDialog> {
        iced::window::oldest()
            .and_then(|id| iced::window::run(id, |w| AsyncFileDialog::new().set_parent(w)))
    }

    pub fn get_games_task(&self) -> Task<Message> {
        let mount_point = self.config.mount_point.clone();
        Task::perform(GameList::new(mount_point), Message::GotGames)
    }

    pub fn download_ui_covers_task(&self) -> Task<Message> {
        let ids = self.games.get_all_game_ids();
        let data_dir = self.data_dir.clone();
        let preferred_language = self.config.preferred_language;

        Task::stream(download_ui_covers(ids, data_dir, preferred_language))
            .map(|_| Message::LoadCovers)
    }

    pub fn get_homebrew_apps_task(&self) -> Task<Message> {
        let mount_point = self.config.mount_point.clone();
        Task::perform(HomebrewAppList::new(mount_point), Message::GotHomebrewApps)
    }

    pub fn get_drive_info_task(&self) -> Task<Message> {
        let mount_point = &self.config.mount_point;
        if mount_point.as_os_str().is_empty() {
            return Task::none();
        }

        let mount_point = mount_point.clone();
        Task::perform(DriveInfo::try_from_path(mount_point), Message::GotDriveInfo)
    }

    pub fn get_disc_info_task(&self, game: Game) -> Task<Message> {
        Task::perform(
            async move {
                let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;
                let mut file = File::open(&disc_path).await?;
                let meta = wii_disc_info::Meta::read_async(&mut file).await?;
                Ok(meta)
            },
            Message::GotDiscInfo,
        )
    }

    pub fn delete_dir_task(&mut self, path: PathBuf) -> Task<Message> {
        self.current_modal = None;

        Task::perform(
            async move { fs::remove_dir_all(path).await.map_err(Into::into) },
            Message::DirDeleted,
        )
    }

    pub fn import_homebrew_apps_task(&self, paths: Vec<PathBuf>) -> Task<Message> {
        let mount_point = self.config.mount_point.clone();

        Task::perform(
            homebrew::import(mount_point, paths, self.config.remove_sources_apps),
            Message::HomebrewAppsImported,
        )
    }

    pub fn import_games_task(&mut self, paths: Vec<PathBuf>) -> Task<Message> {
        self.import_queue.extend(paths);

        if !self.ongoing.contains(Ongoing::Converting) {
            self.status.clear();

            let task = if let Some(path) = self.import_queue.pop() {
                self.ongoing.remove(Ongoing::Converting);

                Task::sip(
                    import_game(path, self.config.clone(), self.drive_info.clone()),
                    Message::SetStatus,
                    Message::GameImported,
                )
            } else {
                self.notifications
                    .add(Notification::info("Import queue is empty"));
                Task::none()
            };

            Task::batch([task, self.get_games_task(), self.get_drive_info_task()])
        } else {
            Task::none()
        }
    }

    pub fn load_covers(&mut self) {
        for game in self.games.iter_mut() {
            if game.cover.is_none() {
                game.load_cover_blocking(&self.data_dir);
            }
        }
    }
}
