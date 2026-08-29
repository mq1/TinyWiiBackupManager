// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game_list::GameList,
    homebrew::homebrew_app_list::HomebrewAppList,
    messages::Message,
    notifications::notification::Notification,
    state::{AppState, Ongoing},
    ui::{dialogs, modals::Modal},
};
use iced::Task;

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(page) => {
                self.current_page = page;
                Task::none()
            }
            Message::PickMountPoint => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_mount_point_dialog_task),
            Message::MountPointPicked(None) => Task::none(),
            Message::MountPointPicked(Some(path)) => {
                self.config.mount_point = path;
                self.write_config_task()
            }
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => {
                self.ongoing.insert(
                    Ongoing::GettingGames
                        | Ongoing::GettingHomebrewApps
                        | Ongoing::GettingDriveInfo,
                );

                Task::batch([
                    self.get_games_task(),
                    self.get_homebrew_apps_task(),
                    self.get_drive_info_task(),
                ])
            }
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
                self.ongoing.remove(Ongoing::GettingGames);
                self.load_covers();
                self.download_ui_covers_task()
            }
            Message::GotGames(Err(e)) => {
                self.games = GameList::default();
                self.notifications.add(Notification::error(e));
                self.ongoing.remove(Ongoing::GettingGames);
                Task::none()
            }
            Message::GotHomebrewApps(Ok(homebrew_apps)) => {
                self.homebrew_apps = homebrew_apps;
                self.ongoing.remove(Ongoing::GettingHomebrewApps);
                Task::none()
            }
            Message::GotHomebrewApps(Err(e)) => {
                self.homebrew_apps = HomebrewAppList::default();
                self.notifications.add(Notification::error(e));
                self.ongoing.remove(Ongoing::GettingHomebrewApps);
                Task::none()
            }
            Message::GotDriveInfo(Ok(drive_info)) => {
                self.drive_info = Some(drive_info);
                self.ongoing.remove(Ongoing::GettingDriveInfo);
                Task::none()
            }
            Message::GotDriveInfo(Err(e)) => {
                self.drive_info = None;
                self.notifications.add(Notification::error(e));
                self.ongoing.remove(Ongoing::GettingDriveInfo);
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
            Message::DirDeleted(Ok(())) => Task::done(Message::RefreshGamesAndApps),
            Message::DirDeleted(Err(e)) => {
                self.notifications.add(Notification::error(e));
                Task::done(Message::RefreshGamesAndApps)
            }
            Message::PickHomebrewApps => self
                .init_file_dialog_task()
                .then(dialogs::make_pick_homebrew_apps_dialog_task),
            Message::ImportHomebrewApps(paths) => self.import_homebrew_apps_task(paths),
            Message::HomebrewAppsImported(Ok(n)) if n > 0 => {
                self.notifications.add(Notification::success(format!(
                    "{n} Homebrew app(s) successfully imported"
                )));

                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::HomebrewAppsImported(Ok(_)) => Task::none(),
            Message::HomebrewAppsImported(Err(e)) => {
                self.notifications.add(Notification::error(e));
                Task::none()
            }
            Message::SetStatus(status) => {
                self.status = status;
                Task::none()
            }
            Message::CalcGameSha1(game) => {
                Task::sip(game.calc_sha1(), Message::SetStatus, Message::GotGameSha1)
            }
            Message::GotGameSha1(Ok(msg)) => {
                self.notifications.add(Notification::success(msg));
                self.status.clear();
                Task::none()
            }
            Message::GotGameSha1(Err(e)) => {
                self.notifications.add(Notification::error(e));
                self.status.clear();
                Task::none()
            }
            Message::PickGames => {
                let existing_ids = self.games.get_all_game_ids();
                self.init_file_dialog_task().then(move |base| {
                    dialogs::make_pick_games_dialog_task(base, existing_ids.clone())
                })
            }
            Message::PickGamesRecursively => {
                let existing_ids = self.games.get_all_game_ids();
                self.init_file_dialog_task().then(move |base| {
                    dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
                })
            }
            Message::ImportGames(paths) => self.import_games_task(paths),
            Message::GameImported(Ok(())) => {
                self.ongoing.remove(Ongoing::Converting);
                self.import_games_task(vec![])
            }
            Message::GameImported(Err(e)) => {
                self.ongoing.remove(Ongoing::Converting);
                self.notifications.add(Notification::error(e));
                self.import_games_task(vec![])
            }
            Message::CancelImport(i) => {
                self.import_queue.remove(i);
                Task::none()
            }
            Message::CancelAllImports => {
                self.import_queue.clear();
                Task::none()
            }
            Message::ToggleAnimationState => {
                if self.ongoing.contains(Ongoing::Converting) {
                    self.ongoing.toggle(Ongoing::AnimationState);
                }
                Task::none()
            }
            Message::LoadCovers => {
                self.load_covers();
                Task::none()
            }
        }
    }
}
