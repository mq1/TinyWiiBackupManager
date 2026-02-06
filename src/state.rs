// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, ThemePreference},
    data_dir::get_data_dir,
    games::{
        archive::ArchiveOperation,
        banners,
        checksum::ChecksumOperation,
        convert_for_wii::ConvertForWiiOperation,
        covers, dir_layout,
        game::Game,
        game_list::{self, GameList},
        id_map,
        strip::StripOperation,
        transfer::{TransferOperation, TransferQueue},
        txtcodes,
        util::maybe_path_to_entry,
        wiitdb::{self},
    },
    hbc::{self, app_list::HbcAppList, osc::OscAppMeta, osc_list::OscAppList, wiiload},
    known_mount_points,
    message::Message,
    notifications::Notifications,
    ui::{Screen, dialogs, lucide, window_color},
    updater,
    util::{DriveInfo, clean_old_files},
};
use iced::{
    Subscription, Task, Theme,
    widget::{
        Id,
        operation::{self, AbsoluteOffset},
    },
    window,
};
use semver::Version;
use std::{ffi::OsStr, path::PathBuf};
use which_fs::FsKind;

#[cfg(target_os = "macos")]
use crate::util::run_dot_clean;

pub struct State {
    pub screen: Screen,
    pub data_dir: PathBuf,
    pub config: Config,
    pub game_list: GameList,
    pub games_filter: String,
    pub hbc_app_list: HbcAppList,
    pub osc_app_list: OscAppList,
    pub notifications: Notifications,
    pub show_wii: bool,
    pub show_gc: bool,
    pub drive_info: Option<DriveInfo>,
    pub osc_filter: String,
    pub hbc_filter: String,
    pub new_version: Option<Version>,
    pub transfer_queue: TransferQueue,
    pub status: String,
    pub manual_archiving_game: Option<PathBuf>,
    pub osc_icons_download_started: bool,

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
        clean_old_files(&data_dir);

        let mut initial_state = Self {
            screen: Screen::Games,
            data_dir,
            config,
            game_list: GameList::empty(),
            games_filter: String::new(),
            hbc_app_list: HbcAppList::empty(),
            osc_app_list: OscAppList::empty(),
            notifications: Notifications::new(),
            show_wii: true,
            show_gc: true,
            drive_info: None,
            osc_filter: String::new(),
            hbc_filter: String::new(),
            new_version: None,
            transfer_queue: TransferQueue::new(),
            status: String::new(),
            manual_archiving_game: None,
            osc_icons_download_started: false,

            // scroll positions
            games_scroll_id: Id::unique(),
            games_scroll_offset: AbsoluteOffset::default(),
            hbc_scroll_id: Id::unique(),
            hbc_scroll_offset: AbsoluteOffset::default(),
            osc_scroll_id: Id::unique(),
            osc_scroll_offset: AbsoluteOffset::default(),
        };

        if known_mount_points::check(&initial_state) {
            initial_state.notifications.info("New drive detected, a path normalization run is recommended\nYou can find it in the Toolbox page".to_string());
        }

        let set_window_color = window::oldest()
            .and_then(move |id| iced::system::theme().map(move |mode| (id, mode)))
            .then(move |(id, mode)| window::run(id, move |w| window_color::set(w, mode)))
            .discard();

        let tasks = Task::batch(vec![
            set_window_color,
            game_list::get_list_games_task(&initial_state),
            id_map::get_init_task(),
            lucide::get_load_lucide_task(),
            DriveInfo::get_task(&initial_state),
            hbc::app_list::get_list_hbc_apps_task(&initial_state),
            hbc::osc_list::get_load_osc_apps_task(&initial_state),
            updater::get_check_update_task(),
        ]);

        (initial_state, tasks)
    }

    pub fn title(&self) -> String {
        if self.config.is_mount_point_valid() {
            format!(
                "TinyWiiBackupManager  ›  {}",
                self.config.mount_point().display()
            )
        } else {
            "TinyWiiBackupManager  ›  No drive selected".to_string()
        }
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.theme_preference() {
            ThemePreference::Light => Some(Theme::Light),
            ThemePreference::Dark => Some(Theme::Dark),
            ThemePreference::System => None,
        }
    }

    pub fn subscription(_: &Self) -> Subscription<Message> {
        iced::event::listen_with(|event, _status, _id| {
            if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
                Some(Message::FileDropped(path))
            } else {
                None
            }
        })
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
            | Message::GotOscAppList(Err(e))
            | Message::GotGameList(Err(e))
            | Message::GotLatestVersion(Err(e))
            | Message::GotHbcAppList(Err(e))
            | Message::DirectoryDeleted(Err(e)) => {
                self.notifications.error(e);
                Task::none()
            }
            Message::NavTo(Screen::Games) => {
                self.screen = Screen::Games;
                operation::scroll_to(self.games_scroll_id.clone(), self.games_scroll_offset)
            }
            Message::NavTo(Screen::GameInfo(mut game)) => {
                let tasks = vec![
                    game.get_load_disc_info_task(),
                    wiitdb::get_get_game_info_task(self, &game),
                ];

                self.screen = Screen::GameInfo(game);
                Task::batch(tasks)
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

                let task1 =
                    operation::scroll_to(self.osc_scroll_id.clone(), self.osc_scroll_offset);

                if self.osc_icons_download_started {
                    task1
                } else {
                    self.osc_icons_download_started = true;
                    let task2 = hbc::osc_list::get_download_icons_task(self);
                    task1.chain(task2)
                }
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
                DriveInfo::get_task(self),
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
                    let new_config = self.config.clone_with_mount_point(mount_point);
                    let _ = self.update(Message::UpdateConfig(new_config));

                    if known_mount_points::check(self) {
                        self.notifications.info("New drive detected, a path normalization run is recommended\nYou can find it in the Toolbox page".to_string());
                    }

                    self.update(Message::RefreshGamesAndApps)
                } else {
                    Task::none()
                }
            }
            Message::AskDeleteDirConfirmation(path) => window::oldest()
                .and_then(move |id| {
                    let path = path.clone();
                    window::run(id, move |id| {
                        dialogs::delete_dir(id, &path)
                            .map_err(|e| format!("Failed to delete directory: {e:#}"))
                    })
                })
                .map(Message::DirectoryDeleted),
            Message::DirectoryDeleted(Ok(())) => {
                let task1 = match &self.screen {
                    Screen::GameInfo(_) => self.update(Message::NavTo(Screen::Games)),
                    Screen::HbcInfo(_) => self.update(Message::NavTo(Screen::HbcApps)),
                    _ => Task::none(),
                };

                let task2 = self.update(Message::RefreshGamesAndApps);

                task1.chain(task2)
            }
            Message::GotOscAppList(Ok(app_list)) => {
                // match installed apps
                for app in self.hbc_app_list.iter_mut() {
                    if let Some(i) = app_list.iter().position(|a| a.name() == app.meta().name()) {
                        app.set_osc_i(i);
                    }
                }

                self.osc_app_list = app_list;

                Task::none()
            }
            Message::UpdateOscFilter(filter) => {
                self.osc_app_list.fuzzy_search(&filter);
                self.osc_filter = filter;
                Task::none()
            }
            Message::GotGameList(Ok(game_list)) => {
                self.game_list = game_list;
                self.game_list.sort(SortBy::None, self.config.sort_by());

                covers::get_cache_cover3ds_task(self)
            }
            Message::GotHbcAppList(Ok(mut app_list)) => {
                // match osc apps
                if !self.osc_app_list.is_empty() {
                    for app in app_list.iter_mut() {
                        if let Some(i) = self
                            .osc_app_list
                            .iter()
                            .position(|a| a.name() == app.meta().name())
                        {
                            app.set_osc_i(i);
                        }
                    }
                }

                self.hbc_app_list = app_list;
                self.hbc_app_list.sort(SortBy::None, self.config.sort_by());

                Task::none()
            }
            Message::GotDriveInfo(drive_info) => {
                self.drive_info = drive_info;
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
                let _ = self.update(Message::GenericResult(res));
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
            Message::UpdateConfig(new_config) => {
                self.config = new_config;
                if let Err(e) = self.config.write() {
                    self.notifications
                        .error(format!("Failed to write config: {e:#}"));
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
                    .info("Downloading wiitdb.xml to drive...".to_string());
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
                let _ = self.update(Message::GenericResult(res));
                self.update(Message::RefreshGamesAndApps)
            }
            Message::ChooseGamesToAdd => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_games))
                .map(Message::ConfirmAddGamesToTransferStack),
            Message::ChooseGamesSrcDir => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_src_dir))
                .map(Message::ConfirmAddGamesToTransferStack),
            Message::ConfirmAddGamesToTransferStack(mut entries) => {
                // remove already installed games
                entries.retain(|(path, id)| {
                    let is_multidisc = path.file_stem().and_then(OsStr::to_str).is_some_and(|s| {
                        let s = s.to_ascii_lowercase();
                        s.contains("disc 1") || s.contains("disc 2")
                    });

                    let is_installed = self.game_list.iter().any(|g| g.id() == *id);

                    is_multidisc || !is_installed
                });

                if entries.is_empty() {
                    window::oldest()
                        .and_then(|id| window::run(id, dialogs::no_new_games))
                        .discard()
                } else {
                    window::oldest()
                        .and_then(move |id| {
                            let entries = entries.clone();
                            window::run(id, move |w| dialogs::confirm_add_games(w, entries))
                        })
                        .map(Message::AddGamesToTransferStack)
                }
            }
            Message::AddGamesToTransferStack((paths, yes)) => {
                if yes {
                    let is_fat32 = self
                        .drive_info
                        .as_ref()
                        .is_some_and(|i| i.fs_kind() == FsKind::Fat32);

                    for path in paths {
                        self.transfer_queue.push(TransferOperation::ConvertForWii(
                            ConvertForWiiOperation::new(path, self.config.clone(), is_fat32),
                        ));
                    }

                    if self.status.is_empty() {
                        self.update(Message::StartTransfer)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::StartTransfer => {
                if let Some(task) = self.transfer_queue.pop_task() {
                    task
                } else {
                    self.notifications
                        .success("Finished all game transfers/conversions!".to_string());
                    Task::none()
                }
            }
            Message::CancelTransfer(i) => {
                self.transfer_queue.cancel(i);
                Task::none()
            }
            Message::Transferred(Ok(msg)) => {
                self.status.clear();

                if let Some(msg) = msg {
                    self.notifications.info(msg);
                }

                Task::batch(vec![
                    self.update(Message::StartTransfer),
                    self.update(Message::RefreshGamesAndApps),
                ])
            }
            Message::Transferred(Err(e)) => {
                self.status.clear();
                self.notifications.error(e);
                self.notifications
                    .error("Aborting queued game operations!".to_string());
                self.transfer_queue.cancel_all();
                self.update(Message::RefreshGamesAndApps)
            }
            Message::GotDiscInfo(res) => {
                if let Screen::GameInfo(game) = &mut self.screen {
                    game.set_disc_info(res.map_err(|e| e.clone()));
                }

                Task::none()
            }
            Message::GotGameInfo(res) => {
                if let Screen::GameInfo(game) = &mut self.screen {
                    game.set_wiitdb_info(res.map_err(|e| e.clone()));
                }

                Task::none()
            }
            #[cfg(target_os = "macos")]
            Message::RunDotClean => {
                use iced::futures::TryFutureExt;

                let mount_point = self.config.mount_point().clone();

                Task::perform(
                    async move { run_dot_clean(mount_point) }
                        .map_err(|e| format!("dot_clean failed: {e:#}")),
                    Message::GenericResult,
                )
            }
            Message::OpenThat(uri) => {
                if let Err(e) = open::that(&uri) {
                    self.notifications
                        .error(format!("Failed to open {}: {:#}", uri.display(), e));
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
                eprintln!("No new version of {} available", env!("CARGO_PKG_NAME"));
                Task::none()
            }
            Message::UpdateTransferStatus(status) => {
                self.status = status;
                Task::none()
            }
            Message::ChooseArchiveDest(source, title) => {
                if let Some(source) = source {
                    window::oldest()
                        .and_then(move |id| {
                            let source = source.clone();
                            let title = title.clone();
                            window::run(id, move |w| dialogs::choose_archive_dest(w, source, title))
                        })
                        .map(Message::ArchiveGame)
                } else {
                    self.notifications
                        .error("Failed to get source path for archive".to_string());
                    Task::none()
                }
            }
            Message::ArchiveGame(None) => Task::none(),
            Message::ArchiveGame(Some((source, title, dest))) => {
                let op = ArchiveOperation::new(source, title, dest);
                self.transfer_queue.push(TransferOperation::Archive(op));

                if self.status.is_empty() {
                    self.update(Message::StartTransfer)
                } else {
                    Task::none()
                }
            }
            Message::DownloadCoversForUsbLoaderGx => {
                self.notifications.info(
                    "Downloading covers for USB Loader GX, this may take some time!".to_string(),
                );
                covers::get_download_all_covers_task(self)
            }
            Message::DownloadCoversForWiiflow => {
                self.notifications
                    .info("Downloading covers for Wiiflow, this may take some time!".to_string());
                covers::get_download_wiiflow_covers_task(self)
            }
            Message::CloseAllNotifications => {
                self.notifications.close_all();
                Task::none()
            }
            Message::DownloadCheatsForGame(game) => {
                self.notifications
                    .info(format!("Downloading cheats for {}...", game.title()));
                txtcodes::get_download_cheats_for_game_task(self, &game)
            }
            Message::DownloadCheatsForAllGames => {
                self.notifications
                    .info("Downloading cheats for all games, this may take some time!".to_string());
                txtcodes::get_download_cheats_for_all_games_task(self)
            }
            Message::DownloadBanners => {
                self.notifications.info(
                    "Downloading banners for all GameCube games, this may take some time!"
                        .to_string(),
                );
                banners::get_download_banners_task(self)
            }
            Message::NormalizePaths => {
                let res = dir_layout::normalize_paths(self.config.mount_point())
                    .map_err(|e| format!("Failed to normalize paths: {e:#}"));
                let _ = self.update(Message::GenericResult(res));
                self.update(Message::RefreshGamesAndApps)
            }
            Message::ChooseFileToWiiload => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_file_to_wiiload))
                .map(Message::Wiiload),
            Message::Wiiload(path) => {
                if let Some(path) = path {
                    self.notifications
                        .info("Sending file to Wii...".to_string());
                    wiiload::get_send_via_wiiload_task(self, path)
                } else {
                    Task::none()
                }
            }
            Message::WiiloadOsc(app) => {
                let zip_url = app.assets().archive().url().clone();
                self.notifications
                    .info("Sending file to Wii...".to_string());
                wiiload::get_download_and_send_via_wiiload_task(self, zip_url)
            }
            Message::ConfirmStripGame(game) => window::oldest()
                .and_then(move |id| {
                    let game = game.clone();
                    window::run(id, move |w| dialogs::confirm_strip_game(w, game))
                })
                .map(Message::StripGame),
            Message::StripGame((game, yes)) => {
                if yes {
                    self.notifications
                        .info(format!("Removing update partition from {}", game.title()));

                    let is_fat32 = self
                        .drive_info
                        .as_ref()
                        .is_some_and(|i| i.fs_kind() == FsKind::Fat32);

                    let op = StripOperation::new(game, self.config.always_split(), is_fat32);
                    self.transfer_queue.push(TransferOperation::Strip(op));

                    if self.status.is_empty() {
                        self.update(Message::StartTransfer)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::ConfirmStripAllGames => window::oldest()
                .and_then(|id| window::run(id, dialogs::confirm_strip_all_games))
                .map(Message::StripAllGames),
            Message::StripAllGames(yes) => {
                if yes {
                    self.notifications.info(
                        "Removing update partition from all games, this may take some time!"
                            .to_string(),
                    );

                    let is_fat32 = self
                        .drive_info
                        .as_ref()
                        .is_some_and(|i| i.fs_kind() == FsKind::Fat32);

                    for game in self.game_list.iter().cloned() {
                        let op = StripOperation::new(game, self.config.always_split(), is_fat32);
                        self.transfer_queue.push(TransferOperation::Strip(op));
                    }

                    if self.status.is_empty() {
                        self.update(Message::StartTransfer)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::ChecksumGame(game) => {
                self.notifications
                    .info(format!("Calculating checksum for {}", game.title()));

                let op = ChecksumOperation::new(game);
                self.transfer_queue.push(TransferOperation::Checksum(op));

                if self.status.is_empty() {
                    self.update(Message::StartTransfer)
                } else {
                    Task::none()
                }
            }
            Message::ChooseGameToArchiveManually => window::oldest()
                .and_then(|id| window::run(id, dialogs::choose_game_to_archive_manually))
                .map(Message::SetManualArchivingGame),
            Message::SetManualArchivingGame(game) => {
                self.manual_archiving_game = game;
                Task::none()
            }
            Message::RunManualGameArchiving => {
                if let Some(path) = self.manual_archiving_game.take()
                    && let Some(title) = path.file_stem().and_then(OsStr::to_str)
                {
                    let title = title.to_string();
                    self.update(Message::ChooseArchiveDest(Some(path), title))
                } else {
                    Task::none()
                }
            }
            Message::FileDropped(path) => match self.screen {
                Screen::Games => {
                    if let Some(entry) = maybe_path_to_entry(path) {
                        self.update(Message::ConfirmAddGamesToTransferStack(vec![entry]))
                    } else {
                        self.notifications.error("Invalid file!".to_string());
                        Task::none()
                    }
                }
                Screen::HbcApps => self.update(Message::AddHbcApps(vec![path])),
                _ => Task::none(),
            },
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
