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
              -> Vec<(Action, SharedString)> {
            let logic = weak.upgrade().unwrap();
            let mut args = args.as_str().split('\0');

            match action {
                Action::NotifyInfo => {
                    let msg = args.next().unwrap();
                    notifications.push(Notification::info(msg));

                    Vec::new()
                }
                Action::NotifyError => {
                    let msg = args.next().unwrap();
                    notifications.push(Notification::error(msg));

                    Vec::new()
                }
                Action::SyncConfig => {
                    logic.set_config(DisplayedConfig::from(&config));

                    if let Err(e) = config.write() {
                        let msg = slint::format!("Failed to write config: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    Vec::new()
                }
                Action::PickMountPoint => {
                    if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                        config.contents.mount_point = path;

                        if config.check_mount_point() {
                            notifications.push(Notification::info(NEW_DRIVE_TEXT));
                        }
                    }

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::ToggleShowWii => {
                    config.contents.show_wii = !config.contents.show_wii;
                    *show_wii.borrow_mut() = config.contents.show_wii;
                    filtered_games.reset();

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::ToggleShowGc => {
                    config.contents.show_gc = !config.contents.show_gc;
                    *show_gc.borrow_mut() = config.contents.show_gc;
                    filtered_games.reset();

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetWiiOutputFormat => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.wii_output_format = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetGcOutputFormat => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.gc_output_format = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetAlwaysSplit => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.always_split = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetScrubUpdatePartition => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.scrub_update_partition = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetRemoveSourcesGames => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.remove_sources_games = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetRemoveSourcesApps => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.remove_sources_apps = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetTxtCodesSource => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.txt_codes_source = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetThemePreference => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.theme_preference = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetViewAs => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.view_as = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetSortBy => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.sort_by = value;
                    *sort_by.borrow_mut() = value;
                    sorted_games.reset();
                    sorted_homebrew_apps.reset();

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::SetPreferredLanguage => {
                    let value = args.next().unwrap().parse().unwrap();
                    config.contents.preferred_language = value;

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::WiiloadLocalFile => {
                    let wii_ip = args.next().unwrap().to_string();

                    if let Some(in_path) = dialogs::pick_wiiload(&window_handle) {
                        let msg = slint::format!("Sending {} to Wii...", in_path.display());
                        notifications.push(Notification::info(msg));

                        config.contents.wii_ip = wii_ip.clone();

                        let weak = weak.clone();
                        std::thread::spawn(move || {
                            let res = twbm_core::wiiload::send(&wii_ip, &in_path);

                            let _ = weak.upgrade_in_event_loop(move |logic| match res {
                                Ok(msg) => logic
                                    .invoke_dispatch(Action::NotifyInfo, msg.to_shared_string()),
                                Err(e) => {
                                    let msg = slint::format!("Could not send file to Wii: {e}");
                                    logic.invoke_dispatch(Action::NotifyError, msg)
                                }
                            });
                        });
                    }

                    vec![(Action::SyncConfig, SharedString::new())]
                }
                Action::WiiloadOscApp => {
                    let wii_ip = args.next().unwrap().to_string();
                    let uid = args.next().unwrap().parse().unwrap();

                    config.contents.wii_ip = wii_ip.clone();

                    let i = osc_apps.binary_search_by_key(&uid, |app| app.uid).unwrap();
                    let app = osc_apps[i].clone();

                    let msg = slint::format!("Sending {} to Wii...", &app.meta.name);
                    notifications.push(Notification::info(msg));

                    let weak = weak.clone();
                    std::thread::spawn(move || {
                        let res = app.wiiload(&wii_ip);

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(msg) => {
                                logic.invoke_dispatch(Action::NotifyInfo, msg.to_shared_string())
                            }
                            Err(e) => {
                                let msg = slint::format!("Could not send file to Wii: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg)
                            }
                        });
                    });

                    vec![(Action::SyncConfig, SharedString::new())]
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

                    vec![(Action::PairHomebrewOsc, SharedString::new())]
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

                    Vec::new()
                }
                Action::OpenThat => {
                    let uri = args.next().unwrap();

                    if let Err(e) = open::that(uri) {
                        let msg = slint::format!("Failed to open URL: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    Vec::new()
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

                    Vec::new()
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

                    Vec::new()
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

                    Vec::new()
                }
                Action::OscContentsCached => {
                    let (new, hours, minutes) =
                        twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

                    let displayed_apps = new.iter().map(DisplayedOscApp::from).collect::<Vec<_>>();

                    osc_apps = new;

                    displayed_osc_apps.set_vec(displayed_apps);
                    logic.set_osc_refreshed_x_hours_ago(hours);
                    logic.set_osc_refreshed_x_minutes_ago(minutes);

                    vec![(Action::PairHomebrewOsc, SharedString::new())]
                }
                Action::ReloadOscIcon => {
                    let i = args.next().unwrap().parse().unwrap();
                    let mut app = displayed_osc_apps.row_data(i).unwrap();
                    let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.slug));

                    if let Ok(icon) = Image::load_from_path(&icon_path) {
                        app.icon = icon;
                        displayed_osc_apps.set_row_data(i, app);
                    }

                    Vec::new()
                }
                Action::FilterGames => {
                    let filter = args.next().unwrap();
                    *games_filter.borrow_mut() = filter.to_shared_string();
                    filtered_games.reset();

                    Vec::new()
                }
                Action::FilterHomebrewApps => {
                    let filter = args.next().unwrap();
                    *homebrew_apps_filter.borrow_mut() = filter.to_shared_string();
                    filtered_homebrew_apps.reset();

                    Vec::new()
                }
                Action::FilterOscApps => {
                    let filter = args.next().unwrap();
                    *osc_apps_filter.borrow_mut() = filter.to_shared_string();
                    filtered_osc_apps.reset();

                    Vec::new()
                }
                Action::CloseNotification => {
                    let i = args.next().unwrap().parse().unwrap();
                    notifications.remove(i);

                    Vec::new()
                }
                Action::Checksum => {
                    let uid = args.next().unwrap().parse().unwrap();

                    let i = games.binary_search_by_key(&uid, |g| g.uid).unwrap();
                    let game = games[i].clone();

                    let weak = weak.clone();

                    std::thread::spawn(move || {
                        let weak2 = weak.clone();
                        let update_progress = move |percentage| {
                            let status = slint::format!("{percentage}%");
                            let _ = weak2.upgrade_in_event_loop(move |logic| {
                                logic.invoke_dispatch(Action::SetCrc32Status, status);
                            });
                        };

                        let res = checksum::perform(game, &update_progress);

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(crc32) => {
                                let status = slint::format!("{crc32:08x}");
                                logic.invoke_dispatch(Action::SetCrc32Status, status);
                            }
                            Err(e) => {
                                let msg = slint::format!("Checksum failed: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                        });
                    });

                    Vec::new()
                }
                Action::PickGames => {
                    let recursively = args.next().unwrap().parse().unwrap();

                    let paths = if recursively {
                        dialogs::pick_games_r(&window_handle)
                    } else {
                        dialogs::pick_games(&window_handle)
                    };

                    let existing_ids = games.iter().map(|g| g.id).collect::<Vec<_>>();

                    let mut new = Vec::new();
                    for path in paths {
                        if let Ok(mut f) = File::open(&path)
                            && let Ok(meta) = wii_disc_info::Meta::read(&mut f)
                            && let Some(game_id) = GameID::new(meta.game_id())
                            && existing_ids.iter().all(|id| *id != game_id)
                        {
                            new.push(QueuedConversion {
                                kind: ConversionKind::Standard,
                                in_path: path.to_string_lossy().to_shared_string(),
                                ..Default::default()
                            });
                        }
                    }

                    conversion_queue_buffer.set_vec(new);

                    Vec::new()
                }
                Action::ConfirmConversionQueueBuffer => {
                    conversion_queue.extend(conversion_queue_buffer.iter());
                    conversion_queue_buffer.clear();

                    if !is_converting {
                        is_converting = true;
                        vec![(Action::TriggerConversion, SharedString::new())]
                    } else {
                        Vec::new()
                    }
                }
                Action::TriggerConversion => {
                    if conversion_queue.row_count() == 0 {
                        is_converting = false;
                        let msg = SharedString::from("Conversion queue empty");
                        notifications.push(Notification::info(msg));
                        return Vec::new();
                    }

                    let queued = conversion_queue.remove(0);
                    let conv = Conversion::new(&queued);

                    let weak = weak.clone();
                    let drive_info = drive_info.clone();
                    let config = config.clone();

                    let _ = std::thread::spawn(move || {
                        conv.perform(config, drive_info, weak);
                    });

                    Vec::new()
                }
                Action::ClearConversionQueueBuffer => {
                    conversion_queue_buffer.clear();

                    Vec::new()
                }
                Action::SetCrc32Status => {
                    let status = args.next().unwrap();
                    logic.set_crc32_status(status.to_shared_string());

                    Vec::new()
                }
                Action::ScrubGame => {
                    let uid = args.next().unwrap().parse().unwrap();
                    let i = games.binary_search_by_key(&uid, |g| g.uid).unwrap();
                    let game = &games[i];

                    let Some(disc_path) = game.get_disc_path() else {
                        let msg = slint::format!("No disc path found for game {}", game.title);
                        notifications.push(Notification::error(msg));
                        return Vec::new();
                    };

                    let conv = QueuedConversion {
                        kind: ConversionKind::Scrub,
                        in_path: disc_path.to_string_lossy().to_shared_string(),
                        game_title: game.title.to_shared_string(),
                        game_id: game.id.to_shared_string(),
                        ..Default::default()
                    };

                    conversion_queue.push(conv);

                    if !is_converting {
                        is_converting = true;
                        vec![(Action::TriggerConversion, SharedString::new())]
                    } else {
                        Vec::new()
                    }
                }
                Action::PickHomebrewApps => {
                    let paths = dialogs::pick_homebrew_apps(&window_handle);

                    let res = twbm_core::util::install_zips(&config.contents.mount_point, &paths);

                    if let Err(e) = res {
                        let msg = slint::format!("Failed to install apps: {e}");
                        notifications.push(Notification::error(msg));
                    } else {
                        let msg = slint::format!("{} apps installed successfully", paths.len());
                        notifications.push(Notification::info(msg));
                    }

                    vec![(Action::RefreshAll, SharedString::new())]
                }
                Action::InstallOscApp => {
                    let uid = args.next().unwrap().parse().unwrap();
                    let i = osc_apps.binary_search_by_key(&uid, |app| app.uid).unwrap();
                    let app = osc_apps[i].clone();

                    let root_dir = config.contents.mount_point.clone();

                    let msg = slint::format!("Installing {}", &app.meta.name);
                    notifications.push(Notification::info(msg));

                    let weak = weak.clone();

                    std::thread::spawn(move || {
                        let res = app.install(&root_dir);

                        let _ = weak.upgrade_in_event_loop(move |logic| {
                            if let Err(e) = res {
                                logic.invoke_dispatch(Action::NotifyError, e.to_shared_string());
                            } else {
                                let msg =
                                    slint::format!("{} installed successfully", &app.meta.name);
                                logic.invoke_dispatch(Action::NotifyInfo, msg);
                            }

                            logic.invoke_dispatch(Action::RefreshAll, SharedString::new());
                        });
                    });

                    Vec::new()
                }
                Action::ReloadCover => {
                    let i = args.next().unwrap().parse().unwrap();
                    let mut game = displayed_games.row_data(i).unwrap();
                    let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

                    if let Ok(cover) = Image::load_from_path(&cover_path) {
                        game.cover = cover;
                        displayed_games.set_row_data(i, game);
                    }

                    Vec::new()
                }
                Action::FinishedDownloadingCovers => {
                    is_downloading_covers = false;

                    Vec::new()
                }
                Action::DeleteGame => {
                    let uid = args.next().unwrap().parse().unwrap();
                    let i = games.binary_search_by_key(&uid, |g| g.uid).unwrap();
                    let game = &games[i];

                    if let Err(e) = fs::remove_dir_all(&game.path) {
                        let msg = slint::format!("Failed to delete game: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    vec![(Action::RefreshAll, SharedString::new())]
                }
                Action::DeleteHomebrewApp => {
                    let uid = args.next().unwrap().parse().unwrap();
                    let i = homebrew_apps
                        .binary_search_by_key(&uid, |app| app.uid)
                        .unwrap();
                    let app = &homebrew_apps[i];

                    if let Err(e) = fs::remove_dir_all(&app.path) {
                        let msg = slint::format!("Failed to delete homebrew app: {e}");
                        notifications.push(Notification::error(msg));
                    }

                    vec![(Action::RefreshAll, SharedString::new())]
                }
                Action::ScrubAllGames => {
                    let to_scrub = games
                        .iter()
                        .filter_map(|game| {
                            if !game.is_wii {
                                return None;
                            }

                            let disc_path = game.get_disc_path()?;
                            let mut f = File::open(disc_path).ok()?;
                            let meta = wii_disc_info::Meta::read(&mut f).ok()?;
                            let worth = meta.format() == wii_disc_info::Format::Wbfs
                                && is_worth_scrubbing(&mut f).ok()?;

                            worth.then_some(game.uid)
                        })
                        .collect::<Vec<_>>();

                    if to_scrub.is_empty() {
                        let msg = "No games need scrubbing";
                        notifications.push(Notification::info(msg));
                        return Vec::new();
                    }

                    to_scrub
                        .iter()
                        .map(|uid| (Action::ScrubGame, uid.to_shared_string()))
                        .collect()
                }
                Action::NormalizeDirLayout => {
                    match normalize_dir_layout::perform(&config.contents.mount_point) {
                        Ok(_) => {
                            let msg = "Directory layout successfully normalized";
                            notifications.push(Notification::info(msg));
                        }
                        Err(e) => {
                            let msg = slint::format!("Failed to normalize directory layout: {e}");
                            notifications.push(Notification::error(msg));
                        }
                    }

                    Vec::new()
                }
                Action::CancelConversion => {
                    let i = args.next().unwrap().parse().unwrap();
                    let _ = conversion_queue.remove(i);

                    Vec::new()
                }
                Action::CancelAllConversions => {
                    conversion_queue.clear();

                    Vec::new()
                }
                Action::DownloadTxtCodes => {
                    let uid = args.next().unwrap().parse().unwrap();
                    let i = games.binary_search_by_key(&uid, |game| game.uid).unwrap();
                    let game_id = games[i].id;

                    let config = config.clone();

                    let msg = slint::format!("Downloading txtcodes for {game_id}");
                    notifications.push(Notification::info(msg));

                    let weak = weak.clone();
                    std::thread::spawn(move || {
                        let res = twbm_core::txtcodes::download_cheats(game_id, &config);

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(_) => {
                                let msg = slint::format!("Downloaded txtcodes for {game_id}");
                                logic.invoke_dispatch(Action::NotifyInfo, msg);
                            }
                            Err(e) => {
                                let msg = slint::format!(
                                    "Failed to download txtcodes for {game_id}: {e}"
                                );
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                        });
                    });

                    Vec::new()
                }
                Action::DownloadAllCovers => {
                    let for_wiiflow: bool = args.next().unwrap().parse().unwrap();
                    let config = config.clone();

                    let ids = games.iter().map(|g| g.id).collect::<Vec<_>>();

                    let msg = if for_wiiflow {
                        "Downloading covers for WiiFlow..."
                    } else {
                        "Downloading covers for USBLoaderGX..."
                    };

                    notifications.push(Notification::info(msg));

                    let weak = weak.clone();
                    let _ = std::thread::spawn(move || {
                        let res = if for_wiiflow {
                            twbm_core::covers::download_all_covers_for_wiiflow(&ids, &config)
                        } else {
                            twbm_core::covers::download_all_covers_for_usbloadergx(&ids, &config)
                        };

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(failed_ids) if failed_ids.is_empty() => {
                                let msg = "All covers downloaded successfully".to_shared_string();
                                logic.invoke_dispatch(Action::NotifyInfo, msg);
                            }
                            Ok(failed_ids) => {
                                let failed_ids = twbm_core::game_id::make_list_string(&failed_ids);
                                let msg = slint::format!(
                                    "Covers downloaded successfully\nThe following games may lack some covers: {failed_ids}"
                                );
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                            Err(e) => {
                                let msg = slint::format!("Failed to download covers: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                        });
                    });

                    Vec::new()
                }
                Action::DownloadAllBanners => {
                    let mount_point = config.contents.mount_point.clone();

                    let ids = games
                        .iter()
                        .filter(|g| !g.is_wii)
                        .map(|g| g.id)
                        .collect::<Vec<_>>();

                    let msg = slint::format!("Downloading banners for {} games", ids.len());
                    notifications.push(Notification::info(msg));

                    let weak = weak.clone();
                    std::thread::spawn(move || {
                        let res = twbm_core::banners::download_banners(&mount_point, &ids);

                        let _ = weak.upgrade_in_event_loop(move |logic| match res {
                            Ok(failed_ids) if failed_ids.is_empty() => {
                                let msg = "All banners downloaded successfully".to_shared_string();
                                logic.invoke_dispatch(Action::NotifyInfo, msg);
                            }
                            Ok(failed_ids) => {
                                let failed_ids = twbm_core::game_id::make_list_string(&failed_ids);
                                let msg = slint::format!(
                                    "Banners downloaded successfully\nExcept the following: {failed_ids}"
                                );
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                            Err(e) => {
                                let msg = slint::format!("Failed to download banners: {e}");
                                logic.invoke_dispatch(Action::NotifyError, msg);
                            }
                        });
                    });

                    Vec::new()
                }
                Action::ArchiveManually => {
                    let Some(in_path) = dialogs::pick_game(&window_handle) else {
                        return Vec::new();
                    };

                    let Some(stem) = in_path.file_stem().and_then(OsStr::to_str) else {
                        return Vec::new();
                    };

                    let Some(out_path) = dialogs::save_game(&window_handle, stem) else {
                        return Vec::new();
                    };

                    let queued = QueuedConversion {
                        kind: ConversionKind::Archive,
                        in_path: in_path.to_string_lossy().to_shared_string(),
                        out_path: out_path.to_string_lossy().to_shared_string(),
                        ..Default::default()
                    };

                    conversion_queue.push(queued);

                    if !is_converting {
                        is_converting = true;
                        return vec![(Action::TriggerConversion, SharedString::new())];
                    }

                    Vec::new()
                }
                Action::SetLatestVersion => {
                    let version = args.next().unwrap();
                    logic.set_latest_version(version.into());

                    Vec::new()
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

                    Vec::new()
                }
                Action::ArchiveGame => {
                    let uid = args.next().unwrap().parse().unwrap();

                    let i = games.binary_search_by_key(&uid, |game| game.uid).unwrap();
                    let game = &games[i];

                    let Some(in_path) = game.get_disc_path() else {
                        let msg = "No disc found for this game!";
                        notifications.push(Notification::error(msg));

                        return Vec::new();
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
                            return vec![(Action::TriggerConversion, SharedString::new())];
                        }
                    }

                    Vec::new()
                }
                Action::CheckMountPoint => {
                    if config.check_mount_point() {
                        notifications.push(Notification::info(NEW_DRIVE_TEXT));
                    }

                    Vec::new()
                }
                Action::SetStatus => {
                    let status = args.next().unwrap();
                    logic.set_status(status.to_shared_string());

                    Vec::new()
                }
                #[cfg(windows)]
                Action::SetWindowColor => {
                    let is_dark = args.next().unwrap().parse().unwrap();
                    crate::window_color::set(is_dark);

                    Vec::new()
                }
                #[cfg(not(windows))]
                Action::SetWindowColor => Vec::new(),
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

                    Vec::new()
                }
                #[cfg(not(target_os = "macos"))]
                Action::RunDotClean => Vec::new(),
            }
        };

        self.on_dispatch(move |action, args| {
            let mut actions = vec![(action, args)];

            while let Some((action, args)) = actions.pop() {
                let mut new_actions = process_action(action, args);
                actions.append(&mut new_actions);
            }
        });

        self.invoke_dispatch(Action::RefreshAll, SharedString::new());
    }
}
