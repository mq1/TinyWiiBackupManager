// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    Action, ConversionKind, DisplayedConfig, DisplayedDiscInfo, DisplayedDriveInfo, DisplayedGame,
    DisplayedHomebrewApp, DisplayedOscApp, Logic, Notification, QueuedConversion,
    convert::Conversion, covers, dialogs, games, homebrew_apps, osc,
};
use slint::{
    FilterModel, Global, Image, Model, ModelRc, SharedString, SortModel, ToSharedString, VecModel,
    Window,
};
use std::{
    cell::RefCell,
    ffi::OsStr,
    fs::{self, File},
    rc::Rc,
    str::Split,
};
use twbm_core::{
    checksum,
    config::Config,
    data_dir::DATA_DIR,
    disc_info::{DiscInfo, is_worth_scrubbing},
    drive_info::DriveInfo,
    game::Game,
    game_id::GameID,
    homebrew_app::HomebrewApp,
    normalize_dir_layout,
    osc::OscApp,
};

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking TWBM update has been installed), a path normalization run is recommended\nYou can find it in the Toolbox page";

impl Logic<'_> {
    pub fn init(&self, mut config: Config, window: &Window) {
        let displayed_config = DisplayedConfig::from(&config);
        let sort_by = Rc::new(RefCell::new(config.contents.sort_by));
        let show_wii = Rc::new(RefCell::new(config.contents.show_wii));
        let show_gc = Rc::new(RefCell::new(config.contents.show_gc));

        let mut games = Vec::<Game>::new();
        let mut homebrew_apps = Vec::<HomebrewApp>::new();
        let mut osc_apps = Vec::<OscApp>::new();
        let mut drive_info = DriveInfo::empty();

        let displayed_games = Rc::new(VecModel::from(Vec::new()));
        let games_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_games = Rc::new(SortModel::new(
            displayed_games.clone(),
            games::get_compare_fn(sort_by.clone()),
        ));
        let filtered_games = Rc::new(FilterModel::new(
            sorted_games.clone(),
            games::get_filter_fn(games_filter.clone(), show_wii.clone(), show_gc.clone()),
        ));

        let displayed_homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_homebrew_apps = Rc::new(SortModel::new(
            displayed_homebrew_apps.clone(),
            homebrew_apps::get_compare_fn(sort_by.clone()),
        ));
        let filtered_homebrew_apps = Rc::new(FilterModel::new(
            sorted_homebrew_apps.clone(),
            homebrew_apps::get_filter_fn(homebrew_apps_filter.clone()),
        ));

        let displayed_osc_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let filtered_osc_apps = Rc::new(FilterModel::new(
            displayed_osc_apps.clone(),
            osc::get_filter_fn(osc_apps_filter.clone()),
        ));

        let conversion_queue = Rc::new(VecModel::from(Vec::new()));
        let conversion_queue_buffer = Rc::new(VecModel::from(Vec::new()));

        let notifications = Rc::new(VecModel::from(Vec::new()));

        let mut is_converting = false;
        let mut is_downloading_osc_icons = false;
        let mut is_downloading_covers = false;

        self.set_app_version(env!("CARGO_PKG_VERSION").to_shared_string());
        self.set_data_dir(DATA_DIR.to_string_lossy().to_shared_string());
        self.set_config(displayed_config);
        self.set_games(ModelRc::from(filtered_games.clone()));
        self.set_homebrew_apps(ModelRc::from(filtered_homebrew_apps.clone()));
        self.set_osc_apps(ModelRc::from(filtered_osc_apps.clone()));
        self.set_notifications(ModelRc::from(notifications.clone()));
        self.set_conversion_queue(ModelRc::from(conversion_queue.clone()));
        self.set_conversion_queue_buffer(ModelRc::from(conversion_queue_buffer.clone()));

        // Mutations

        let weak = self.as_weak();
        let window_handle = window.window_handle();
        let mut process_action = move |action: Action,
                                       args: SharedString|
              -> Option<(Action, SharedString)> {
            let logic = weak.upgrade().unwrap();
            let args = args.split('\0');

            match action {
                Action::NotifyInfo => {
                    let msg = args.next().unwrap();
                    notifications.push(Notification::info(msg));

                    None
                }
                Action::NotifyError => {
                    let msg = args.next().unwrap();
                    notifications.push(Notification::error(msg));

                    None
                }
                Action::SyncConfig => {
                    logic.set_config(DisplayedConfig::from(&config));

                    if let Err(e) = config.write() {
                        let msg = slint::format!("Failed to write config: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    None
                }
                Action::PickMountPoint => {
                    if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                        config.contents.mount_point = path;

                        if config.check_mount_point() {
                            notifications.push(Notification::info(NEW_DRIVE_TEXT));
                        }
                    }

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::ToggleShowWii => {
                    config.contents.show_wii = !config.contents.show_wii;
                    *show_wii.borrow_mut() = config.contents.show_wii;
                    filtered_games.reset();

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::ToggleShowGc => {
                    config.contents.show_gc = !config.contents.show_gc;
                    *show_gc.borrow_mut() = config.contents.show_gc;
                    filtered_games.reset();

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetWiiOutputFormat => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.wii_output_format = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetGcOutputFormat => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.gc_output_format = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetAlwaysSplit => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.always_split = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetScrubUpdatePartition => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.scrub_update_partition = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetRemoveSourcesGames => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.remove_sources_games = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetRemoveSourcesApps => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.remove_sources_apps = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetTxtCodesSource => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.txt_codes_source = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetThemePreference => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.theme_preference = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetViewAs => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.view_as = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetSortBy => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.sort_by = value;
                    *sort_by.borrow_mut() = value;
                    sorted_games.reset();
                    sorted_homebrew_apps.reset();

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::SetPreferredLanguage => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.preferred_language = value;

                    Some((Action::SyncConfig, SharedString::new()))
                }
                Action::RefreshAll => {
                    let root_path = &config.contents.mount_point;

                    let new_games = games::scan_drive(root_path);
                    let new_apps = homebrew_apps::scan_drive(root_path);
                    let new_drive_info =
                        DriveInfo::from_path(root_path).unwrap_or(DriveInfo::empty());

                    let ids = new_games.iter().map(|g| g.id).collect::<Vec<_>>();

                    let new_displayed_games = new_games
                        .iter()
                        .map(DisplayedGame::from)
                        .collect::<Vec<_>>();

                    let new_displayed_apps = new_apps
                        .iter()
                        .map(DisplayedHomebrewApp::from)
                        .collect::<Vec<_>>();

                    let new_displayed_drive_info = DisplayedDriveInfo::from(&new_drive_info);

                    games = new_games;
                    homebrew_apps = new_apps;
                    drive_info = new_drive_info;

                    displayed_games.set_vec(new_displayed_games);
                    displayed_homebrew_apps.set_vec(new_displayed_apps);
                    logic.set_drive_info(new_displayed_drive_info);

                    if !is_downloading_covers {
                        is_downloading_covers = true;

                        let weak = weak.clone();
                        let preferred_language = config.contents.preferred_language;

                        let _ = std::thread::spawn(move || {
                            let res = covers::download_covers(ids, preferred_language, &weak);

                            if let Err(e) = res {
                                let _ = weak.upgrade_in_event_loop(move |logic| {
                                    let msg = slint::format!("Could not download covers: {e}");
                                    logic.invoke_dispatch(Action::NotifyError, msg);
                                });
                            }
                        });
                    }

                    Some((Action::PairHomebrewOsc, SharedString::new()))
                }
                Action::PairHomebrewOsc => {
                    let mut displayed_apps = homebrew_apps
                        .iter()
                        .map(DisplayedHomebrewApp::from)
                        .collect::<Vec<_>>();

                    for app in &mut displayed_apps {
                        if let Some(osc_app) = osc_apps
                            .iter()
                            .find(|osc_app| osc_app.meta.name.as_str() == app.name.as_str())
                        {
                            app.osc_app = DisplayedOscApp::from(osc_app);
                        }
                    }

                    displayed_homebrew_apps.set_vec(displayed_apps);

                    None
                }
                Action::OpenThat => {
                    let uri = args.next().unwrap();

                    if let Err(e) = open::that(uri) {
                        let msg = slint::format!("Failed to open URL: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    None
                }
                Action::DownloadOscIcons => {
                    if !is_downloading_osc_icons {
                        is_downloading_osc_icons = true;

                        let weak = weak.clone();
                        let apps = osc_apps.clone();

                        let _ = std::thread::spawn(move || {
                            osc::download_icons(&apps, weak);
                        });
                    }

                    None
                }
                Action::CheckForUpdates => {
                    let weak = weak.clone();

                    std::thread::spawn(move || {
                        let res = twbm_core::updates::check();

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(Some(version)) => {
                                let value = slint::format!("v{version}");
                                logic.invoke_dispatch(Action::SetLatestVersion, value);
                            }
                            Ok(None) => {
                                eprintln!("No updates available");
                            }
                            Err(e) => {
                                let msg = slint::format!("Failed to check for updates: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                        });
                    });

                    None
                }
                Action::CacheOscContents => {
                    let force_refresh = args.next().unwrap().parse().unwrap();
                    let weak = weak.clone();

                    std::thread::spawn(move || {
                        let res = twbm_core::osc::cache_contents(&DATA_DIR, force_refresh);

                        let _ = weak.upgrade_in_event_loop(|logic| {
                            if let Err(e) = res {
                                let msg = slint::format!("Failed to cache OSC contents: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            } else {
                                logic.invoke_dispatch(
                                    Action::OscContentsCached,
                                    SharedString::new(),
                                );
                            }
                        });
                    });

                    None
                }
                Action::OscContentsCached => {
                    let (new, hours, minutes) =
                        twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

                    let displayed_apps = new.iter().map(DisplayedOscApp::from).collect::<Vec<_>>();

                    osc_apps = new;

                    displayed_osc_apps.set_vec(displayed_apps);
                    logic.set_osc_refreshed_x_hours_ago(hours);
                    logic.set_osc_refreshed_x_minutes_ago(minutes);

                    Some((Action::PairHomebrewOsc, SharedString::new()))
                }
                Action::CloseNotification => {
                    let i = args.next().unwrap().parse().unwrap();
                    notifications.remove(i);

                    None
                }
                Action::LoadGameInfo => {
                    let uid = args.next().unwrap().parse().unwrap();

                    let i = games.binary_search_by_key(&uid, |game| game.uid).unwrap();
                    let game = &games[i];

                    if let Some(disc_path) = game.get_disc_path()
                        && let Some(info) = DiscInfo::from_path(disc_path)
                    {
                        let info = DisplayedDiscInfo::from(&info);
                        logic.set_current_disc_info(info);
                    }

                    None
                }
                Action::ArchiveGame => {
                    let uid = args.next().unwrap().parse().unwrap();

                    let i = games.binary_search_by_key(&uid, |game| game.uid).unwrap();
                    let game = &games[i];

                    let Some(in_path) = game.get_disc_path() else {
                        let msg = "No disc found for this game!";
                        notifications.push(Notification::error(msg));

                        return None;
                    };

                    let out_path = dialogs::save_game(&window_handle, &game.title);

                    if let Some(out_path) = out_path {
                        let queued = QueuedConversion {
                            kind: ConversionKind::Archive,
                            in_path: in_path.to_string_lossy().to_shared_string(),
                            out_path: out_path.to_string_lossy().to_shared_string(),
                            ..Default::default()
                        };

                        conversion_queue.push(queued);

                        if !is_converting {
                            is_converting = true;
                            return Some((Action::TriggerConversion, SharedString::new()));
                        }
                    }

                    None
                }
                Action::CheckMountPoint => {
                    if config.check_mount_point() {
                        notifications.push(Notification::info(NEW_DRIVE_TEXT));
                    }

                    None
                }
                Action::SetStatus => {
                    let status = args.next().unwrap();
                    logic.set_status(status.to_shared_string());

                    None
                }
                #[cfg(windows)]
                Action::SetWindowColor => {
                    let is_dark = args.next().unwrap().parse().unwrap();
                    crate::window_color::set(is_dark);

                    None
                }
                #[cfg(not(windows))]
                Action::SetWindowColor => None,
                #[cfg(target_os = "macos")]
                Action::RunDotClean => {
                    let res = {
                        let root_path = &config.contents.mount_point;
                        twbm_core::util::run_dot_clean(root_path)
                    };

                    match res {
                        Ok(_) => {
                            let msg = "Successfully ran dot_clean";
                            notifications.push(Notification::info(msg));
                        }
                        Err(e) => {
                            let msg = slint::format!("Failed to run dot_clean: {e}");
                            notifications.push(Notification::error(msg));
                        }
                    }

                    None
                }
                #[cfg(not(target_os = "macos"))]
                Action::RunDotClean => None,
            }
        };

        self.on_dispatch(move |action, args| {
            let mut action = Some((action, args));
            while let Some((next_action, args)) = action {
                action = process_action(next_action, args);
            }
        });
    }
}
