// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    data_dir::get_data_dir,
    game::{self, Game},
    game_id::GameID,
    message::Message,
    notifications::Notifications,
    ui::{Screen, dialogs},
    wiitdb,
};
use iced::{
    Task,
    window::{self},
};
use std::path::PathBuf;

pub struct State {
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
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let data_dir = get_data_dir().expect("Failed to get data dir");
        let config = Config::load(&data_dir);

        let games = game::list(config.get_drive_path(), &None);

        let initial_state = Self {
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
        };

        let task = wiitdb::get_load_wiitdb_task(&initial_state);

        (initial_state, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }
            Message::RefreshGames => {
                self.games = game::list(self.config.get_drive_path(), &self.wiitdb);
                Task::none()
            }
            Message::RefreshHbcApps => {
                self.hbc_apps.clear();
                Task::none()
            }
            Message::OpenProjectRepo => {
                let _ = open::that(env!("CARGO_PKG_REPOSITORY"));
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
            Message::SelectMountPoint => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_mount_point))
                .map(Message::MountPointChosen),
            Message::MountPointChosen(mount_point) => {
                if let Some(mount_point) = mount_point {
                    if let Err(e) = self.config.update_drive_path(mount_point) {
                        self.notifications.error(e.to_string());
                    } else {
                        return self.update(Message::RefreshGames);
                    }
                }
                Task::none()
            }
            Message::AskDeleteGame(i) => {
                let game = &self.games[i];
                let title = game.title.clone();

                window::oldest()
                    .and_then(move |id| {
                        let title = title.clone();
                        window::run(id, move |w| dialogs::delete_game(w, title))
                    })
                    .map(move |yes| Message::DeleteGame(i, yes))
            }
            Message::DeleteGame(i, yes) => {
                if yes {
                    if let Err(e) = self.games[i].delete() {
                        self.notifications.error(e.to_string());
                    } else {
                        return self.update(Message::RefreshGames);
                    }
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
