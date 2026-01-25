// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, ThemePreference},
    data_dir::get_data_dir,
    games::{
        archive::ArchiveOperation,
        convert_for_wii::ConvertForWiiOperation,
        covers,
        game::Game,
        game_list::{self, GameList},
        transfer::{TransferOperation, TransferQueue},
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
    widget::{
        Id,
        operation::{self, AbsoluteOffset},
    },
    window,
};
use semver::Version;
use std::{path::PathBuf, sync::Arc};

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
    pub new_version: Option<Version>,
    pub transfer_queue: TransferQueue,
    pub status: String,

    // scroll positions
    pub games_scroll_id: Id,
    pub games_scroll_offset: AbsoluteOffset,
    pub hbc_scroll_id: Id,
    pub hbc_scroll_offset: AbsoluteOffset,
    pub osc_scroll_id: Id,
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
            new_version: None,
            transfer_queue: TransferQueue::new(),
            status: String::new(),

            // scroll positions
            games_scroll_id: Id::unique(),
            games_scroll_offset: AbsoluteOffset::default(),
            hbc_scroll_id: Id::unique(),
            hbc_scroll_offset: AbsoluteOffset::default(),
            osc_scroll_id: Id::unique(),
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
            Message::GenericResult(Ok(s)) | Message::GenericSuccess(s) => {
                self.notifications.success(s);
                Task::none()
            }
            Message::EmptyResult(Ok(())) => Task::none(),
            Message::GenericResult(Err(e))
            | Message::EmptyResult(Err(e))
            | Message::GenericError(e)
            | Message::GotWiitdbDatafile(Err(e))
            | Message::GotOscAppList(Err(e))
            | Message::GotGameList(Err(e))
            | Message::GotLatestVersion(Err(e))
            | Message::GotDiscInfo(Err(e))
            | Message::GotHbcAppList(Err(e))
            | Message::Transferred(Err(e)) => {
                self.notifications.error(e);
                Task::none()
            }
            Message::NavTo(Screen::Games) => {
                self.screen = Screen::Games;
                operation::scroll_to(self.games_scroll_id.clone(), self.games_scroll_offset)
            }
            Message::NavTo(Screen::GameInfo(mut game)) => {
                if let Some(wiitdb) = &self.wiitdb {
                    game.update_wiitdb_info(wiitdb);
                }

                let task = game.get_load_disc_info_task();
                self.screen = Screen::GameInfo(game);
                task
            }
            Message::NavTo(Screen::HbcApps) => {
                self.screen = Screen::HbcApps;
                operation::scroll_to(self.hbc_scroll_id.clone(), self.hbc_scroll_offset)
            }
            Message::NavTo(Screen::HbcInfo(app)) => {
                self.screen = Screen::HbcInfo(app);
                Task::none()
            }
            Message::NavTo(Screen::Osc) => {
                self.screen = Screen::Osc;
                operation::scroll_to(self.osc_scroll_id.clone(), self.osc_scroll_offset)
            }
            Message::NavTo(Screen::OscInfo(app)) => {
                self.screen = Screen::OscInfo(app);
                Task::none()
            }
            Message::NavTo(Screen::Toolbox) => {
                self.screen = Screen::Toolbox;
                Task::none()
            }
            Message::NavTo(Screen::Settings) => {
                self.screen = Screen::Settings;
                Task::none()
            }
            Message::NavTo(Screen::Transfer) => {
                self.screen = Screen::Transfer;
                Task::none()
            }
            Message::NavTo(Screen::About) => {
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
            Message::UpdateScrollPosition(id, offset) => {
                if id == self.games_scroll_id {
                    self.games_scroll_offset = offset;
                } else if id == self.hbc_scroll_id {
                    self.hbc_scroll_offset = offset;
                } else if id == self.osc_scroll_id {
                    self.osc_scroll_offset = offset;
                }

                Task::none()
            }
            Message::GotWiitdbDatafile(Ok((wiitdb, downloaded))) => {
                for game in self.game_list.iter_mut() {
                    game.update_title(&wiitdb);
                }

                let sort_by = self.config.sort_by();
                if matches!(sort_by, SortBy::NameAscending | SortBy::NameDescending) {
                    self.game_list.sort(SortBy::None, sort_by);
                }

                self.wiitdb = Some(wiitdb);

                if downloaded {
                    self.notifications
                        .info("GameTDB Datafile (wiitdb.xml) downloaded successfully");
                }

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
                .and_then(move |id| {
                    let path = path.clone();
                    window::run(id, move |id| {
                        dialogs::delete_dir(id, &path).map_err(Arc::new)
                    })
                })
                .map(Message::DirectoryDeleted),
            Message::DirectoryDeleted(res) => {
                let mut tasks = vec![
                    self.update(Message::EmptyResult(res)),
                    self.update(Message::RefreshGamesAndApps),
                ];

                if let Screen::GameInfo(_) = &self.screen {
                    tasks.push(self.update(Message::NavTo(Screen::Games)));
                } else if let Screen::HbcInfo(_) = &self.screen {
                    tasks.push(self.update(Message::NavTo(Screen::HbcApps)));
                }

                Task::batch(tasks)
            }
            Message::GotOscAppList(Ok(app_list)) => {
                self.osc_app_list = app_list;
                hbc::osc_list::get_download_icons_task(self)
            }
            Message::UpdateOscFilter(filter) => {
                self.osc_app_list.fuzzy_search(&filter);
                self.osc_filter = filter;
                Task::none()
            }
            Message::GotGameList(Ok(game_list)) => {
                self.game_list = game_list;

                if let Some(wiitdb) = &self.wiitdb {
                    for game in self.game_list.iter_mut() {
                        game.update_title(wiitdb);
                    }
                }

                self.game_list.sort(SortBy::None, self.config.sort_by());

                covers::get_cache_cover3ds_task(self)
            }
            Message::GotHbcAppList(Ok(app_list)) => {
                self.hbc_app_list = app_list;
                self.hbc_app_list.sort(SortBy::None, self.config.sort_by());
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
            Message::AppInstalled(res) => Task::batch(vec![
                self.update(Message::GenericResult(res)),
                self.update(Message::RefreshGamesAndApps),
            ]),
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
            Message::UpdateConfig(new_config) => {
                self.config = new_config;
                if let Err(e) = self.config.write() {
                    self.notifications.error(Arc::new(e));
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
            Message::HbcAppsInstalled(res) => Task::batch(vec![
                self.update(Message::GenericResult(res)),
                self.update(Message::RefreshGamesAndApps),
            ]),
            Message::ChooseGamesToAdd => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_games))
                .map(Message::AddGamesToTransferStack),
            Message::ChooseGamesSrcDir => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_src_dir))
                .map(Message::AddGamesToTransferStack),
            Message::AddGamesToTransferStack(paths) => {
                if paths.is_empty() {
                    Task::none()
                } else {
                    let had_pending_operations = self.transfer_queue.has_pending_operations();
                    let operations = paths
                        .into_iter()
                        .map(|p| {
                            TransferOperation::ConvertForWii(ConvertForWiiOperation::new(
                                p,
                                self.config.clone(),
                            ))
                        })
                        .collect();

                    self.transfer_queue.push_multiple(operations);

                    if had_pending_operations {
                        Task::none()
                    } else {
                        self.update(Message::StartTransfer)
                    }
                }
            }
            Message::StartTransfer => {
                if let Some(task) = self.transfer_queue.pop_task() {
                    task
                } else {
                    self.notifications
                        .success("Finished transferring all games");
                    Task::none()
                }
            }
            Message::CancelTransfer(i) => {
                self.transfer_queue.cancel(i);
                Task::none()
            }
            Message::Transferred(Ok(msg)) => {
                self.notifications.success(msg);
                self.status.clear();

                Task::batch(vec![
                    self.update(Message::StartTransfer),
                    self.update(Message::RefreshGamesAndApps),
                ])
            }
            Message::GotDiscInfo(Ok(disc_info)) => {
                if let Screen::GameInfo(game) = &mut self.screen {
                    game.update_disc_info(disc_info);
                }

                Task::none()
            }
            #[cfg(target_os = "macos")]
            Message::RunDotClean => {
                use iced::futures::TryFutureExt;
                use std::sync::Arc;

                let mount_point = self.config.mount_point().clone();

                Task::perform(
                    async move { util::run_dot_clean(mount_point) }.map_err(Arc::new),
                    Message::GenericResult,
                )
            }
            Message::OpenThat(uri) => {
                if let Err(e) = open::that(&uri) {
                    self.notifications.error(Arc::new(anyhow::Error::from(e)));
                }
                Task::none()
            }
            Message::GotLatestVersion(Ok(Some(version))) => {
                self.notifications.info(format!(
                    "A new version of {} is available: {}",
                    env!("CARGO_PKG_NAME"),
                    version
                ));

                self.new_version = Some(version);
                Task::none()
            }
            Message::GotLatestVersion(Ok(None)) => {
                println!("No new version of {} available", env!("CARGO_PKG_NAME"));
                Task::none()
            }
            Message::UpdateTransferStatus(status) => {
                self.status = status;
                Task::none()
            }
            Message::ChooseArchiveDest(game) => window::oldest()
                .and_then(move |id| {
                    let game = game.clone();
                    window::run(id, move |w| dialogs::choose_archive_dest(w, game))
                })
                .map(Message::ArchiveGame),
            Message::ArchiveGame(None) => Task::none(),
            Message::ArchiveGame(Some((game, dest))) => {
                let op = ArchiveOperation::new(game, dest);
                self.transfer_queue.push(TransferOperation::Archive(op));
                self.update(Message::StartTransfer)
            }
            Message::DownloadCoversForUsbLoaderGx => {
                self.notifications
                    .info("Downloading covers for USB Loader GX, this may take some time!");
                covers::get_download_all_covers_task(self)
            }
            Message::DownloadCoversForWiiflow => {
                self.notifications
                    .info("Downloading covers for Wiiflow, this may take some time!");
                covers::get_download_wiiflow_covers_task(self)
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
        let icon_path = icons_dir.join(app.slug()).with_extension("png");

        if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        }
    }
}
