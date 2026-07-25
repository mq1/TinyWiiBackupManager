// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, notifications::Notification, state::AppState, ui::components::Modal,
};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NoOp => Task::none(),
            Message::NavigateTo(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::PickMountPoint => self.pick_mount_point_task(),
            Message::MountPointPicked(path) => {
                self.config.contents.mount_point = path;
                self.write_config_task()
            }
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
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
                    Task::batch([self.get_games_task(), self.get_drive_info_task()])
                } else {
                    Task::none()
                }
            }
            Message::GotGames(games) => {
                self.games = games;
                Task::none()
            }
            Message::CouldNotGetGames(e) => {
                self.games.clear();
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::GotDriveInfo(drive_info) => {
                self.drive_info = Some(drive_info);
                Task::none()
            }
            Message::CouldNotGetDriveInfo(e) => {
                self.drive_info = None;
                self.notifications.push(Notification::error(e));
                Task::none()
            }
            Message::Open(url) => {
                if let Err(e) = open::that(url) {
                    self.notifications.push(Notification::error(e.to_string()));
                }

                Task::none()
            }
            Message::OpenGameInfo(idx) => {
                self.current_modal = Some(Modal::GameInfo((idx, None)));
                self.get_disc_info_task(idx)
            }
            Message::CloseModal => {
                self.current_modal = None;
                Task::none()
            }
            Message::GotDiscInfo(new_meta) => {
                if let Some(Modal::GameInfo((_, meta))) = &mut self.current_modal {
                    *meta = Some(new_meta);
                }

                Task::none()
            }
        }
    }
}
