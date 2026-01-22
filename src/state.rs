// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, ThemePreference},
    data_dir::get_data_dir,
    games::{
        covers,
        game::Game,
        game_list::{self, GameList},
        wiitdb::{self, Datafile},
    },
    hbc::{self, app_list::HbcAppList, osc::OscAppMeta, osc_list::OscAppList},
    message::Message,
    notifications::Notifications,
    ui::{Screen, dialogs, lucide},
    updater, util,
};
use iced::{
    Task, Theme,
    widget::operation::{self, AbsoluteOffset},
    window,
};
use semver::Version;
use std::{path::PathBuf, time::Duration};

pub struct State {
    pub screen: Screen,
    pub data_dir: PathBuf,
    pub config: Config,
    pub game_list: GameList,
    pub games_filter: String,
    pub hbc_app_list: HbcAppList,
    pub osc_app_list: OscAppList,
    pub wiitdb: Option<Datafile>,
    pub notifications: Notifications,
    pub show_wii: bool,
    pub show_gc: bool,
    pub drive_usage: String,
    pub osc_filter: String,
    pub hbc_filter: String,
    pub transfer_stack: Vec<PathBuf>,
    #[allow(clippy::struct_field_names)]
    pub half_sec_anim_state: bool,
    pub new_version: Option<Version>,
    pub games_scroll_offset: AbsoluteOffset,
    pub hbc_scroll_offset: AbsoluteOffset,
    pub osc_scroll_offset: AbsoluteOffset,
}

impl State {
    pub fn new() -> (Self, Task<Message>) {
        let data_dir = get_data_dir().expect("Failed to get data dir");
        let config = Config::load(&data_dir);

        let initial_state = Self {
            screen: Screen::Games,
            data_dir,
            config,
            game_list: GameList::empty(),
            games_filter: String::new(),
            hbc_app_list: HbcAppList::empty(),
            osc_app_list: OscAppList::empty(),
            wiitdb: None,
            notifications: Notifications::new(),
            show_wii: true,
            show_gc: true,
            drive_usage: String::new(),
            osc_filter: String::new(),
            hbc_filter: String::new(),
            transfer_stack: Vec::new(),
            half_sec_anim_state: false,
            new_version: None,
            games_scroll_offset: AbsoluteOffset::default(),
            hbc_scroll_offset: AbsoluteOffset::default(),
            osc_scroll_offset: AbsoluteOffset::default(),
        };

        let tasks = Task::batch(vec![
            wiitdb::get_load_wiitdb_task(&initial_state),
            game_list::get_list_games_task(&initial_state),
            hbc::app_list::get_list_hbc_apps_task(&initial_state),
            hbc::osc_list::get_load_osc_apps_task(&initial_state),
            util::get_drive_usage_task(&initial_state),
            lucide::get_load_lucide_task(),
            updater::get_check_update_task(),
        ]);

        (initial_state, tasks)
    }

    pub fn title(&self) -> String {
        format!(
            "TinyWiiBackupManager  ›  {}",
            self.config.mount_point().display()
        )
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.theme_preference() {
            ThemePreference::Light => Some(Theme::Light),
            ThemePreference::Dark => Some(Theme::Dark),
            ThemePreference::System => None,
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GenericResult(res) => {
                match res {
                    Ok(s) => self.notifications.success(s),
                    Err(e) => self.notifications.error(e),
                }
                Task::none()
            }
            Message::NavToGames => {
                self.screen = Screen::Games;
                operation::scroll_to("games_scroll", self.games_scroll_offset)
            }
            Message::NavToGameInfo(mut game) => {
                if let Some(wiitdb) = &self.wiitdb {
                    game.update_wiitdb_info(wiitdb);
                }

                let task = game.get_load_disc_info_task();
                self.screen = Screen::GameInfo(game);
                task
            }
            Message::NavToHbcApps => {
                self.screen = Screen::HbcApps;
                operation::scroll_to("hbc_scroll", self.hbc_scroll_offset)
            }
            Message::NavToHbcAppInfo(app) => {
                self.screen = Screen::HbcInfo(app);
                Task::none()
            }
            Message::NavToOscApps => {
                self.screen = Screen::Osc;
                operation::scroll_to("osc_scroll", self.osc_scroll_offset)
            }
            Message::NavToOscAppInfo(app) => {
                self.screen = Screen::OscInfo(app);
                Task::none()
            }
            Message::NavToToolbox => {
                self.screen = Screen::Toolbox;
                Task::none()
            }
            Message::NavToSettings => {
                self.screen = Screen::Settings;
                Task::none()
            }
            Message::NavToTransfer => {
                self.screen = Screen::Transfer;
                Task::none()
            }
            Message::NavToAbout => {
                self.screen = Screen::About;
                Task::none()
            }
            Message::RefreshGamesAndApps => Task::batch(vec![
                game_list::get_list_games_task(self),
                hbc::app_list::get_list_hbc_apps_task(self),
                util::get_drive_usage_task(self),
            ]),
            Message::UpdateGamesFilter(filter) => {
                self.game_list.fuzzy_search(&filter);
                self.games_filter = filter;
                Task::none()
            }
            Message::UpdateGamesScrollOffset(viewport) => {
                self.games_scroll_offset = viewport.absolute_offset();
                Task::none()
            }
            Message::UpdateHbcScrollOffset(viewport) => {
                self.hbc_scroll_offset = viewport.absolute_offset();
                Task::none()
            }
            Message::UpdateOscScrollOffset(viewport) => {
                self.osc_scroll_offset = viewport.absolute_offset();
                Task::none()
            }
            Message::GotWiitdbDatafile(res) => {
                match res {
                    Ok(wiitdb) => {
                        for game in self.game_list.iter_mut() {
                            game.update_title(&wiitdb);
                        }

                        let sort_by = self.config.sort_by();
                        if matches!(sort_by, SortBy::NameAscending | SortBy::NameDescending) {
                            self.game_list.sort(SortBy::None, sort_by);
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
                self.half_sec_anim_state = !self.half_sec_anim_state;
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
                let mut tasks = Vec::new();
                if let Some(mount_point) = mount_point {
                    let new_config = self.config.clone_with_mount_point(mount_point);

                    tasks.push(self.update(Message::UpdateConfig(new_config)));
                    tasks.push(self.update(Message::RefreshGamesAndApps));
                }

                Task::batch(tasks)
            }
            Message::AskDeleteDirConfirmation(path) => window::oldest()
                .and_then(move |id| window::run(id, dialogs::delete_dir(path.clone())))
                .map(Message::DirectoryDeleted),
            Message::DirectoryDeleted(res) => {
                match self.screen {
                    Screen::GameInfo(_) => self.screen = Screen::Games,
                    Screen::HbcInfo(_) => self.screen = Screen::HbcApps,
                    Screen::OscInfo(_) => self.screen = Screen::Osc,
                    _ => {}
                }

                if let Err(e) = res {
                    self.notifications.error(e);
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::GotOscAppList(res) => match res {
                Ok(osc_app_list) => {
                    self.osc_app_list = osc_app_list;
                    hbc::osc_list::get_download_icons_task(self)
                }
                Err(e) => {
                    self.notifications.error(e);
                    Task::none()
                }
            },
            Message::UpdateOscFilter(filter) => {
                self.osc_app_list.fuzzy_search(&filter);
                self.osc_filter = filter;
                Task::none()
            }
            Message::GotGameList(res) => match res {
                Ok(game_list) => {
                    self.game_list = game_list;

                    if let Some(wiitdb) = &self.wiitdb {
                        for game in self.game_list.iter_mut() {
                            game.update_title(wiitdb);
                        }
                    }

                    self.game_list.sort(SortBy::None, self.config.sort_by());

                    covers::get_cache_cover3ds_task(self)
                }
                Err(e) => {
                    self.notifications.error(e);
                    Task::none()
                }
            },
            Message::GotHbcAppList(res) => {
                match res {
                    Ok(hbc_app_list) => {
                        self.hbc_app_list = hbc_app_list;
                        self.hbc_app_list.sort(SortBy::None, self.config.sort_by());
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
            Message::AskInstallOscApp(app) => window::oldest()
                .and_then(move |id| {
                    let app = app.clone();
                    window::run(id, move |w| dialogs::confirm_install_osc_app(w, app))
                })
                .map(Message::InstallOscApp),
            Message::InstallOscApp((app, yes)) => {
                if yes {
                    let base_dir = self.config.mount_point().clone();
                    app.get_install_task(base_dir)
                } else {
                    Task::none()
                }
            }
            Message::AppInstalled(res) => {
                match res {
                    Ok(name) => {
                        self.notifications.success(format!("App installed: {name}"));
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::UpdateHbcFilter(filter) => {
                self.hbc_app_list.fuzzy_search(&filter);
                self.hbc_filter = filter;
                Task::none()
            }
            Message::ChangeTheme => {
                let new_theme_pref = match self.config.theme_preference() {
                    ThemePreference::Light => ThemePreference::Dark,
                    ThemePreference::Dark => ThemePreference::System,
                    ThemePreference::System => ThemePreference::Light,
                };

                let new_config = self.config.clone_with_theme_preference(new_theme_pref);
                self.update(Message::UpdateConfig(new_config))
            }
            Message::EmptyResult(res) => {
                if let Err(e) = res {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::UpdateConfig(new_config) => {
                self.config = new_config;
                if let Err(e) = self.config.write() {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::SortGamesAndApps(sort_by) => {
                let prev_sort_by = self.config.sort_by();
                self.game_list.sort(prev_sort_by, sort_by);
                self.hbc_app_list.sort(prev_sort_by, sort_by);

                let new_config = self.config.clone_with_sort_by(sort_by);
                self.update(Message::UpdateConfig(new_config))
            }
            Message::DownloadWiitdbToDrive => {
                self.notifications
                    .info("Downloading wiitdb.xml to drive...");
                wiitdb::get_download_wiitdb_to_drive_task(self)
            }
            Message::ChooseHbcAppsToAdd => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_hbc_apps))
                .map(Message::AddHbcApps),
            Message::AddHbcApps(apps) => {
                if apps.is_empty() {
                    Task::none()
                } else {
                    hbc::app::get_install_hbc_apps_task(self, apps)
                }
            }
            Message::HbcAppsInstalled(res) => {
                if let Err(e) = res {
                    self.notifications.error(e);
                } else {
                    self.notifications.success("Apps installed");
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::ChooseGamesToAdd => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_games))
                .map(Message::AddGamesToTransferStack),
            Message::ChooseGamesSrcDir => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_src_dir))
                .map(Message::AddGamesToTransferStack),
            Message::AddGamesToTransferStack(mut paths) => {
                if paths.is_empty() {
                    Task::none()
                } else {
                    let empty = self.transfer_stack.is_empty();
                    self.transfer_stack.append(&mut paths);

                    if empty {
                        self.update(Message::StartSingleGameTransfer)
                    } else {
                        Task::none()
                    }
                }
            }
            Message::StartSingleGameTransfer => {
                // TODO

                if let Some(path) = self.transfer_stack.pop() {
                    Task::perform(
                        async move {
                            // TODO
                            smol::Timer::after(Duration::from_secs(5)).await;
                            Ok(path.to_string_lossy().to_string())
                        },
                        Message::FinishedTransferringSingleGame,
                    )
                } else {
                    self.notifications
                        .success("Finished transferring all games");
                    Task::none()
                }
            }
            Message::FinishedTransferringSingleGame(res) => match res {
                Ok(name) => {
                    self.notifications.info(format!("Transferred: {name}"));
                    self.update(Message::StartSingleGameTransfer)
                }
                Err(e) => {
                    self.notifications.error(e);
                    Task::none()
                }
            },
            Message::CancelTransfer(i) => {
                self.transfer_stack.remove(i);
                Task::none()
            }
            Message::GotDiscInfo(res) => {
                match res {
                    Ok(disc_info) => {
                        if let Screen::GameInfo(game) = &mut self.screen {
                            game.update_disc_info(disc_info);
                        }
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                Task::none()
            }
            #[cfg(target_os = "macos")]
            Message::RunDotClean => {
                let mount_point = self.config.mount_point().clone();

                Task::perform(
                    async move {
                        match util::run_dot_clean(mount_point).await {
                            Ok(()) => Ok("dot_clean successful".to_string()),
                            Err(e) => Err(e.to_string()),
                        }
                    },
                    Message::GenericResult,
                )
            }
            Message::OpenThat(uri) => {
                if let Err(e) = open::that(&uri) {
                    self.notifications.error(e);
                }
                Task::none()
            }
            Message::GotLatestVersion(res) => {
                match res {
                    Ok(Some(version)) => {
                        self.notifications.info(format!(
                            "A new version of {} is available: {}",
                            env!("CARGO_PKG_NAME"),
                            version
                        ));

                        self.new_version = Some(version);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        self.notifications
                            .error(format!("Failed to check for updates: {e}"));
                    }
                }
                Task::none()
            }
        }
    }

    pub fn get_game_cover(&self, game: &Game) -> Option<PathBuf> {
        let covers_dir = self.data_dir.join("covers");
        let cover_path = covers_dir.join(game.id().as_str()).with_extension("png");

        if cover_path.exists() {
            Some(cover_path)
        } else {
            None
        }
    }

    pub fn get_osc_app_icon(&self, app: &OscAppMeta) -> Option<PathBuf> {
        let icons_dir = self.data_dir.join("osc-icons");
        let icon_path = icons_dir.join(&app.slug).with_extension("png");

        if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        }
    }
}
