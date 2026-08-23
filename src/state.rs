// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    errors::Error,
    games::{game::Game, game_list::GameList, import::import_game},
    homebrew::{self, homebrew_app_list::HomebrewAppList},
    messages::Message,
    notifications::{notification::Notification, notification_list::NotificationList},
    ui::{modals::Modal, pages::Page},
    util::drive_info::DriveInfo,
};
use iced::{Task, futures::TryFutureExt};
use rfd::AsyncFileDialog;
use smol::fs::{self, File};
use std::{path::PathBuf, sync::Arc};

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
    pub(crate) is_busy: bool, // if we're converting
    pub(crate) is_getting_games: bool,
    pub(crate) is_getting_homebrew_apps: bool,
    pub(crate) is_getting_drive_info: bool,
}

impl AppState {
    pub fn boot(data_dir: PathBuf) -> impl Fn() -> (Self, Task<Message>) {
        move || {
            let state = Self {
                data_dir: data_dir.clone(),
                ..Default::default()
            };

            let task = state.load_config_task();

            (state, task)
        }
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
        let data_dir = self.data_dir.clone();
        let mount_point = self.config.mount_point.clone();

        Task::perform(GameList::new(data_dir, mount_point), Message::GotGames)
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

    pub fn get_disc_info_task(&self, game: Arc<Game>) -> Task<Message> {
        Task::perform(
            async move {
                let disc_path = game.get_disc_path().await.ok_or(Error::DiscNotFound)?;
                let mut file = File::open(&disc_path).await?;
                let meta = wii_disc_info::Meta::read(&mut file).await?;
                Ok(meta)
            },
            Message::GotDiscInfo,
        )
    }

    pub fn delete_dir_task(&mut self, path: PathBuf) -> Task<Message> {
        self.current_modal = None;

        Task::perform(
            fs::remove_dir_all(path).map_err(Into::into),
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

        if !self.is_busy {
            self.status.clear();

            let task = if let Some(path) = self.import_queue.pop() {
                self.is_busy = true;

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
}
