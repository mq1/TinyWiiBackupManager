// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game_list::GameList, homebrew::homebrew_app_list::HomebrewAppList, messages::Message,
    notifications::Notification, state::AppState, ui::modals::Modal,
};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::PickMountPoint => self.pick_mount_point_task(),
            Message::MountPointPicked(None) => Task::none(),
            Message::MountPointPicked(Some(path)) => {
                self.config.contents.mount_point = path;
                self.write_config_task()
            }
            Message::CloseNotification(idx) => {
                self.notifications.remove(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => self.get_games_task(),
            Message::GotConfig(config) => {
                let new_mount_point =
                    config.contents.mount_point != self.config.contents.mount_point;

                self.config = config;

                if new_mount_point {
                    Task::batch([
                        self.get_games_task(),
                        self.get_homebrew_apps_task(),
                        self.get_drive_info_task(),
                    ])
                } else {
                    Task::none()
                }
            }
            Message::GotGames(Ok(games)) => {
                self.games = games.sorted_by(self.config.contents.sort_by);
                Task::none()
            }
            Message::GotGames(Err(e)) => {
                self.games = GameList::default();
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::GotHomebrewApps(Ok(homebrew_apps)) => {
                self.homebrew_apps = homebrew_apps.sorted_by(self.config.contents.sort_by);
                Task::none()
            }
            Message::GotHomebrewApps(Err(e)) => {
                self.homebrew_apps = HomebrewAppList::default();
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::GotDriveInfo(Ok(drive_info)) => {
                self.drive_info = Some(drive_info);
                Task::none()
            }
            Message::GotDriveInfo(Err(e)) => {
                self.drive_info = None;
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::Open(url) => {
                if let Err(e) = open::that(url) {
                    self.notifications.push(Notification::error(e));
                }

                Task::none()
            }
            Message::OpenGameInfo(idx) => {
                self.current_modal = Some(Modal::GameInfo((idx, None)));
                self.get_disc_info_task(idx)
            }
            Message::OpenHomebrewAppInfo(idx) => {
                self.current_modal = Some(Modal::HomebrewAppInfo(idx));
                Task::none()
            }
            Message::CloseModal => {
                self.current_modal = None;
                Task::none()
            }
            Message::GotDiscInfo(Ok(new_meta)) => {
                if let Some(Modal::GameInfo((_, meta))) = &mut self.current_modal {
                    *meta = Some(new_meta);
                }

                Task::none()
            }
            Message::GotDiscInfo(Err(e)) => {
                if let Some(Modal::GameInfo((_, meta))) = &mut self.current_modal {
                    *meta = None;
                }

                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::WroteConfig(Ok(())) => Task::none(),
            Message::WroteConfig(Err(e)) => {
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::SetViewAs(view_as) => {
                self.config.contents.view_as = view_as;
                self.write_config_task()
            }
            Message::AskDeleteGame(idx) => {
                let path = self.games[idx].path.clone();
                self.current_modal = Some(Modal::DeleteDir(path));
                Task::none()
            }
            Message::AskDeleteHomebrewApp(idx) => {
                let path = self.homebrew_apps[idx].path.clone();
                self.current_modal = Some(Modal::DeleteDir(path));
                Task::none()
            }
            Message::DeleteDir(path) => self.delete_dir_task(path),
            Message::DirDeleted(Ok(())) => Task::none(),
            Message::DirDeleted(Err(e)) => {
                self.notifications.push(Notification::error(e));
                Task::none()
            }
        }
    }
}
