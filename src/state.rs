// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, ThemePreference},
    data_dir::get_data_dir,
    game::{self, Game, Games},
    game_id::GameID,
    hbc::{self, HbcApp},
    message::Message,
    notifications::Notifications,
    osc::{self, OscApp},
    ui::{Screen, dialogs},
    util, wiitdb,
};
use iced::{Task, Theme, font, window};
use iced_fonts::LUCIDE_FONT_BYTES;
use std::path::PathBuf;

pub struct State {
    pub screen: Screen,
    pub data_dir: PathBuf,
    pub config: Config,
    pub games: Box<[Game]>,
    pub games_filter: String,
    pub hbc_apps: Box<[HbcApp]>,
    pub osc_apps: Box<[OscApp]>,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub notifications: Notifications,
    pub show_wii: bool,
    pub show_gc: bool,
    pub drive_usage: String,
    pub osc_filter: String,
    pub hbc_filter: String,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let data_dir = get_data_dir().expect("Failed to get data dir");
        let config = Config::load(&data_dir);

        let initial_state = Self {
            screen: Screen::Games,
            data_dir,
            config,
            games: Box::new([]),
            games_filter: String::new(),
            hbc_apps: Box::new([]),
            osc_apps: Box::new([]),
            wiitdb: None,
            notifications: Notifications::new(),
            show_wii: true,
            show_gc: true,
            drive_usage: String::new(),
            osc_filter: String::new(),
            hbc_filter: String::new(),
        };

        let tasks = Task::batch(vec![
            wiitdb::get_load_wiitdb_task(&initial_state),
            game::get_list_games_task(&initial_state),
            hbc::get_list_hbc_apps_task(&initial_state),
            osc::get_load_osc_apps_task(&initial_state),
            util::get_drive_usage_task(&initial_state),
            font::load(LUCIDE_FONT_BYTES).map(Message::FontLoaded),
        ]);

        (initial_state, tasks)
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
            Screen::OscInfo(_) => "TinyWiiBackupManager • OSC App Info".to_string(),
            Screen::GameInfo(_) => "TinyWiiBackupManager • Game Info".to_string(),
            Screen::About => "TinyWiiBackupManager • About".to_string(),
        }
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.get_theme_pref() {
            ThemePreference::Light => Some(Theme::Light),
            ThemePreference::Dark => Some(Theme::Dark),
            ThemePreference::System => None,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NavigateTo(screen) => {
                self.screen = screen;
                Task::none()
            }
            Message::RefreshGamesAndApps => Task::batch(vec![
                game::get_list_games_task(self),
                hbc::get_list_hbc_apps_task(self),
                util::get_drive_usage_task(self),
            ]),
            Message::OpenProjectRepo => {
                if let Err(e) = open::that(env!("CARGO_PKG_REPOSITORY")) {
                    self.notifications.error(e);
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
                            .info("GameTDB Datafile (wiitdb.xml) loaded successfully");
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
                        self.notifications.error(e);
                    } else {
                        return self.update(Message::RefreshGamesAndApps);
                    }
                }
                Task::none()
            }
            Message::AskDeleteGame(i) => {
                let title = self.games[i].title.clone();

                window::oldest()
                    .and_then(move |id| {
                        let title = title.clone();
                        window::run(id, move |w| dialogs::delete_game(w, title))
                    })
                    .map(move |yes| Message::DeleteGame(i, yes))
            }
            Message::DeleteGame(i, yes) => {
                if yes {
                    self.games[i].get_delete_task()
                } else {
                    Task::none()
                }
            }
            Message::GameDeleted(res) => {
                match res {
                    Ok(title) => {
                        self.notifications.info(format!("Game deleted: {}", title));
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::OpenGameDir(game_i) => {
                if let Err(e) = self.games[game_i].open_dir() {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::GotOscApps(res) => {
                match res {
                    Ok(osc_apps) => {
                        self.osc_apps = osc_apps;
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }
                Task::none()
            }
            Message::OpenGameTdb(game_i) => {
                if let Err(e) = self.games[game_i].open_gametdb() {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::UpdateOscFilter(filter) => {
                self.osc_filter = filter;
                Task::none()
            }
            Message::FontLoaded(res) => {
                if res.is_err() {
                    self.notifications.error("Failed to load lucide icons");
                }
                Task::none()
            }
            Message::OpenOscPage(osc_i) => {
                if let Err(e) = self.osc_apps[osc_i].open_page() {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::GotGames(res) => {
                match res {
                    Ok(games) => {
                        self.games = games;

                        if let Some(wiitdb) = &self.wiitdb {
                            for game in &mut self.games {
                                if let Some(title) = wiitdb.get_title(game.id) {
                                    game.title = title;
                                }
                            }
                        }

                        self.games.sort(SortBy::None, self.config.get_sort_by());
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }
                Task::none()
            }
            Message::GotHbcApps(res) => {
                match res {
                    Ok(hbc_apps) => {
                        self.hbc_apps = hbc_apps;
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }
                Task::none()
            }
            Message::GotDriveUsage(usage) => {
                self.drive_usage = usage;
                Task::none()
            }
            Message::AskInstallOscApp(i) => {
                let name = self.osc_apps[i].meta.name.clone();

                window::oldest()
                    .and_then(move |id| {
                        let name = name.clone();
                        window::run(id, move |w| dialogs::confirm_install_osc_app(w, name))
                    })
                    .map(move |yes| Message::InstallOscApp(i, yes))
            }
            Message::InstallOscApp(usize, yes) => {
                if yes {
                    let base_dir = self.config.get_drive_path().to_path_buf();
                    self.osc_apps[usize].get_install_task(base_dir)
                } else {
                    Task::none()
                }
            }
            Message::AppInstalled(res) => {
                match res {
                    Ok(name) => {
                        self.notifications.info(format!("App installed: {}", name));
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::UpdateHbcFilter(filter) => {
                self.hbc_filter = filter;
                Task::none()
            }
            Message::ChangeTheme => {
                let new_theme_pref = match self.config.get_theme_pref() {
                    ThemePreference::Light => ThemePreference::Dark,
                    ThemePreference::Dark => ThemePreference::System,
                    ThemePreference::System => ThemePreference::Light,
                };

                if let Err(e) = self.config.update_theme_pref(new_theme_pref) {
                    self.notifications.error(e);
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

    pub fn get_osc_app_icon(&self, app: &OscApp) -> Option<PathBuf> {
        let icons_dir = self.data_dir.join("osc-icons");
        let icon_path = icons_dir.join(&app.meta.slug).with_extension("png");

        if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        }
    }
}
