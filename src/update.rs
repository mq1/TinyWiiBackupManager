// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game_list::GameList, homebrew::homebrew_app_list::HomebrewAppList, messages::Message,
    notifications::notification::Notification, state::AppState, ui::modals::Modal,
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
                self.config.mount_point = path;
                self.write_config_task()
            }
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => Task::batch([
                self.get_games_task(),
                self.get_homebrew_apps_task(),
                self.get_drive_info_task(),
            ]),
            Message::GotConfig(config) => {
                let new_mount_point = config.mount_point != self.config.mount_point;

                self.config = config;

                if new_mount_point {
                    Task::done(Message::RefreshGamesAndApps)
                } else {
                    Task::none()
                }
            }
            Message::GotGames(Ok(games)) => {
                self.games = games;
                Task::none()
            }
            Message::GotGames(Err(e)) => {
                self.games = GameList::default();
                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::GotHomebrewApps(Ok(homebrew_apps)) => {
                self.homebrew_apps = homebrew_apps;
                Task::none()
            }
            Message::GotHomebrewApps(Err(e)) => {
                self.homebrew_apps = HomebrewAppList::default();
                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::GotDriveInfo(Ok(drive_info)) => {
                self.drive_info = Some(drive_info);
                Task::none()
            }
            Message::GotDriveInfo(Err(e)) => {
                self.drive_info = None;
                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::Open(url) => {
                if let Err(e) = open::that(url) {
                    self.notifications.add(Notification::error(e));
                }

                Task::none()
            }
            Message::OpenGameInfo(game) => {
                self.current_modal = Some(Modal::GameInfo((game.clone(), None)));
                self.get_disc_info_task(game)
            }
            Message::OpenHomebrewAppInfo(app) => {
                self.current_modal = Some(Modal::HomebrewAppInfo(app));
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

                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::WroteConfig(Ok(())) => Task::none(),
            Message::WroteConfig(Err(e)) => {
                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::SetViewAs(view_as) => {
                self.config.view_as = view_as;
                self.write_config_task()
            }
            Message::AskDeleteDir(path) => {
                self.current_modal = Some(Modal::DeleteDir(path));
                Task::none()
            }
            Message::DeleteDir(path) => self.delete_dir_task(path),
            Message::DirDeleted(Ok(())) => Task::none(),
            Message::DirDeleted(Err(e)) => {
                self.notifications.add(Notification::error(e));
                Task::none()
            }
        }
    }
}
