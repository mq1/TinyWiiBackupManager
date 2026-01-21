// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, ThemePreference},
    covers,
    data_dir::get_data_dir,
    game::Game,
    game_list::{self, GameList},
    hbc::{self, HbcApp, HbcApps},
    lucide,
    message::Message,
    notifications::Notifications,
    osc::{self, OscAppMeta},
    ui::{Screen, dialogs},
    util::{self, FuzzySearchable},
    wiitdb,
};
use iced::{
    Task, Theme,
    futures::{FutureExt, TryFutureExt},
    widget::operation::{self, AbsoluteOffset},
    window,
};
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

pub struct State {
    pub screen: Screen,
    pub data_dir: PathBuf,
    pub config: Config,
    pub game_list: GameList,
    pub games_filter: String,
    pub hbc_apps: Box<[HbcApp]>,
    pub osc_apps: Box<[OscAppMeta]>,
    pub wiitdb: Option<wiitdb::Datafile>,
    pub notifications: Notifications,
    pub show_wii: bool,
    pub show_gc: bool,
    pub drive_usage: String,
    pub osc_filter: String,
    pub filtered_osc_indices: Box<[usize]>,
    pub hbc_filter: String,
    pub filtered_hbc_indices: Box<[usize]>,
    pub transfer_stack: Vec<PathBuf>,
    pub half_sec_anim_state: bool,
    pub scroll_offsets: BTreeMap<Screen, AbsoluteOffset>,
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
            hbc_apps: Box::new([]),
            osc_apps: Box::new([]),
            wiitdb: None,
            notifications: Notifications::new(),
            show_wii: true,
            show_gc: true,
            drive_usage: String::new(),
            osc_filter: String::new(),
            hbc_filter: String::new(),
            transfer_stack: Vec::new(),
            half_sec_anim_state: false,
            filtered_osc_indices: Box::new([]),
            filtered_hbc_indices: Box::new([]),
            scroll_offsets: BTreeMap::new(),
        };

        let tasks = Task::batch(vec![
            wiitdb::get_load_wiitdb_task(&initial_state),
            game_list::get_list_games_task(&initial_state),
            hbc::get_list_hbc_apps_task(&initial_state),
            osc::get_load_osc_apps_task(&initial_state),
            util::get_drive_usage_task(&initial_state),
            lucide::get_load_lucide_task(),
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

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GenericResult(res) => {
                match res {
                    Ok(s) => self.notifications.success(s),
                    Err(e) => self.notifications.error(e),
                }
                Task::none()
            }
            Message::NavigateTo(screen) => {
                let task = if let Some(offset) = self.scroll_offsets.get(&screen) {
                    operation::scroll_to(screen.get_scroll_id(), *offset)
                } else {
                    Task::none()
                };

                self.screen = screen;
                task
            }
            Message::OpenGameInfo(i) => {
                let game = self.game_list.get_unchecked_mut(i);

                if let Some(wiitdb) = &self.wiitdb {
                    game.update_wiitdb_info(wiitdb);
                }

                self.screen = Screen::GameInfo(i);
                game.get_load_disc_info_task(i)
            }
            Message::RefreshGamesAndApps => Task::batch(vec![
                game_list::get_list_games_task(self),
                hbc::get_list_hbc_apps_task(self),
                util::get_drive_usage_task(self),
            ]),
            Message::UpdateGamesFilter(filter) => {
                self.game_list.fuzzy_search(&filter);
                self.games_filter = filter;
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
            Message::GotOscApps(res) => match res {
                Ok(osc_apps) => {
                    self.osc_apps = osc_apps;
                    osc::get_download_icons_task(self)
                }
                Err(e) => {
                    self.notifications.error(e);
                    Task::none()
                }
            },
            Message::UpdateOscFilter(filter) => {
                self.filtered_osc_indices = self.osc_apps.fuzzy_search(&filter);
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
            Message::GotHbcApps(res) => {
                match res {
                    Ok(hbc_apps) => {
                        self.hbc_apps = hbc_apps;
                        self.hbc_apps.sort(SortBy::None, self.config.sort_by());
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
                let name = self.osc_apps[i].name.clone();

                window::oldest()
                    .and_then(move |id| {
                        let name = name.clone();
                        window::run(id, move |w| dialogs::confirm_install_osc_app(w, name))
                    })
                    .map(move |yes| Message::InstallOscApp(i, yes))
            }
            Message::InstallOscApp(usize, yes) => {
                if yes {
                    let base_dir = self.config.mount_point().to_path_buf();
                    self.osc_apps[usize].get_install_task(base_dir)
                } else {
                    Task::none()
                }
            }
            Message::AppInstalled(res) => {
                match res {
                    Ok(name) => {
                        self.notifications
                            .success(format!("App installed: {}", name));
                    }
                    Err(e) => {
                        self.notifications.error(e);
                    }
                }

                self.update(Message::RefreshGamesAndApps)
            }
            Message::UpdateHbcFilter(filter) => {
                self.filtered_hbc_indices = self.hbc_apps.fuzzy_search(&filter);
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
                self.hbc_apps.sort(prev_sort_by, sort_by);

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
                    hbc::get_install_hbc_apps_task(self, apps)
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
                if !paths.is_empty() {
                    let empty = self.transfer_stack.is_empty();
                    self.transfer_stack.append(&mut paths);

                    if empty {
                        self.update(Message::StartSingleGameTransfer)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
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
                    self.notifications.info(format!("Transferred: {}", name));
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
            Message::GotDiscInfo(i, res) => {
                self.game_list.get_unchecked_mut(i).update_disc_info(res);
                Task::none()
            }
            #[cfg(target_os = "macos")]
            Message::RunDotClean => {
                let mount_point = self.config.mount_point().to_path_buf();

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
            Message::UpdateScrollOffset(screen, offset) => {
                self.scroll_offsets.insert(screen, offset);
                Task::none()
            }
            Message::OpenThat(uri) => {
                if let Err(e) = open::that(&uri) {
                    self.notifications.error(e);
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
