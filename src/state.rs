// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    data_dir::get_data_dir,
    game::{self, Game, Games},
    game_id::GameID,
    message::Message,
    notifications::Notifications,
    ui::{Screen, dialogs},
    util, wiitdb,
};
use iced::{Task, window};
use std::path::PathBuf;

pub struct State {
    pub window_id: window::Id,
    pub screen: Screen,
    pub data_dir: PathBuf,
    pub config: Config,
    pub games: Box<[Game]>,
    pub games_filter: String,
    pub hbc_apps: Vec<()>,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub notifications: Notifications,
    pub show_wii: bool,
    pub show_gc: bool,
    pub drive_usage: String,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let window_id = serde_plain::from_str("1").expect("Failed to parse window id");

        let data_dir = get_data_dir().expect("Failed to get data dir");
        let config = Config::load(&data_dir);

        let drive_path = config.get_drive_path();
        let games = game::list(drive_path, &None);
        let drive_usage = util::get_drive_usage(drive_path);

        let initial_state = Self {
            window_id,
            screen: Screen::Games,
            data_dir,
            config,
            games,
            games_filter: String::new(),
            hbc_apps: Vec::new(),
            wiitdb: None,
            notifications: Notifications::new(),
            show_wii: true,
            show_gc: true,
            drive_usage,
        };

        let task = wiitdb::get_load_wiitdb_task(&initial_state);

        (initial_state, task)
    }

    pub fn title(&self) -> String {
        match self.screen {
            Screen::Games => format!(
                "TinyWiiBackupManager • Games • {} ({})",
                self.config.get_drive_path_str(),
                &self.drive_usage
            ),
            Screen::HbcApps => format!(
                "TinyWiiBackupManager • HBC Apps • {} ({})",
                self.config.get_drive_path_str(),
                &self.drive_usage
            ),
            Screen::Osc => "TinyWiiBackupManager • Open Shop Channel".to_string(),
            Screen::GameInfo(_) => "TinyWiiBackupManager • Game Info".to_string(),
            Screen::About => "TinyWiiBackupManager • About".to_string(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }
            Message::RefreshGamesAndApps => {
                let drive_path = self.config.get_drive_path();

                self.games = game::list(drive_path, &self.wiitdb);
                self.games.sort(SortBy::None, self.config.get_sort_by());
                self.hbc_apps.clear(); // TODO
                self.drive_usage = util::get_drive_usage(drive_path);

                Task::none()
            }
            Message::OpenProjectRepo => {
                if let Err(e) = open::that(env!("CARGO_PKG_REPOSITORY")) {
                    self.notifications.error(e.to_string());
                }
                Task::none()
            }
            Message::UpdateGamesFilter(new) => {
                self.games_filter = new;
                Task::none()
            }
            Message::GotWiitdbDatafile(res) => {
                match res {
                    Ok(wiitdb) => {
                        for game in &mut self.games {
                            if let Some(title) = wiitdb.get_title(game.id) {
                                game.title = title;
                            }
                        }

                        let sort_by = self.config.get_sort_by();
                        if matches!(sort_by, SortBy::NameAscending | SortBy::NameDescending) {
                            self.games.sort(SortBy::None, sort_by);
                        }

                        self.wiitdb = Some(wiitdb);
                        self.notifications
                            .info("GameTDB Datafile (wiitdb.xml) loaded successfully".to_string());
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                Task::none()
            }
            Message::NotificationTick => {
                self.notifications.tick();
                Task::none()
            }
            Message::CloseNotification(id) => {
                self.notifications.close(id);
                Task::none()
            }
            Message::ShowWii(show) => {
                self.show_wii = show;
                Task::none()
            }
            Message::ShowGc(show) => {
                self.show_gc = show;
                Task::none()
            }
            Message::SelectMountPoint => window::run(self.window_id, dialogs::choose_mount_point)
                .map(Message::MountPointChosen),
            Message::MountPointChosen(mount_point) => {
                if let Some(mount_point) = mount_point {
                    if let Err(e) = self.config.update_drive_path(mount_point) {
                        self.notifications.error(e.to_string());
                    } else {
                        return self.update(Message::RefreshGamesAndApps);
                    }
                }
                Task::none()
            }
            Message::AskDeleteGame(i) => {
                let title = self.games[i].title.clone();

                window::run(self.window_id, move |w| dialogs::delete_game(w, title))
                    .map(move |yes| Message::DeleteGame(i, yes))
            }
            Message::DeleteGame(i, yes) => {
                if yes {
                    if let Err(e) = self.games[i].delete() {
                        self.notifications.error(e.to_string());
                    } else {
                        return self.update(Message::RefreshGamesAndApps);
                    }
                }
                Task::none()
            }
            Message::OpenGameDir(game_i) => {
                if let Err(e) = self.games[game_i].open_dir() {
                    self.notifications.error(e.to_string());
                }
                Task::none()
            }
            Message::OpenPath(path) => {
                if let Err(e) = open::that(path) {
                    self.notifications.error(e.to_string());
                }
                Task::none()
            }
        }
    }

    pub fn get_game_cover(&self, game: &Game) -> Option<PathBuf> {
        let covers_dir = self.data_dir.join("covers");
        let cover_path = covers_dir.join(game.id.as_str()).with_extension("png");

        if cover_path.exists() {
            Some(cover_path)
        } else {
            None
        }
    }
}
