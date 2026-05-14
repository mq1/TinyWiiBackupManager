// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, Dispatcher, DisplayedConfig, DisplayedDiscInfo, DisplayedDriveInfo, DisplayedGame,
    DisplayedHomebrewApp, DisplayedOscApp, Message, Notification, UiState,
    convert::perform_conversion, covers, dialogs, games, homebrew_apps, osc, state::State,
};
use slint::{ComponentHandle, Image, Model, SharedString, ToSharedString, Weak};
use smallvec::SmallVec;
use std::{
    ffi::OsStr,
    fs::{self, File},
    path::{Path, PathBuf},
};
use twbm_core::{
    checksum,
    conversion_queue::QueuedConversion,
    data_dir::DATA_DIR,
    disc_info::{DiscInfo, is_worth_scrubbing},
    drive_info::DriveInfo,
    game_id::GameID,
    normalize_dir_layout,
};

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking TWBM update has been installed), a path normalization run is recommended\nYou can find it in the Toolbox page";

pub fn update<SG, SH, FG, FH, FO, const N: usize>(
    state: &mut State<SG, SH, FG, FH, FO>,
    weak: &Weak<AppWindow>,
    message: Message,
    args: SharedString,
    message_queue: &mut SmallVec<[(Message, SharedString); N]>,
) where
    SG: FnMut(&DisplayedGame, &DisplayedGame) -> std::cmp::Ordering + 'static,
    SH: FnMut(&DisplayedHomebrewApp, &DisplayedHomebrewApp) -> std::cmp::Ordering + 'static,
    FG: Fn(&DisplayedGame) -> bool + 'static,
    FH: Fn(&DisplayedHomebrewApp) -> bool + 'static,
    FO: Fn(&DisplayedOscApp) -> bool + 'static,
{
    let mut args = args.split('\0');

    match message {
        Message::NotifyInfo => {
            let text = args.next().unwrap();
            state.notifications.push(Notification::info(text));
        }
        Message::NotifyError => {
            let text = args.next().unwrap();
            state.notifications.push(Notification::error(text));
        }
        Message::SyncConfig => {
            let app = weak.upgrade().unwrap();

            app.global::<UiState<'_>>()
                .set_config(DisplayedConfig::from(&state.config));

            if let Err(e) = state.config.write() {
                let text = slint::format!("Failed to write config: {e}");
                state.notifications.push(Notification::error(text));
            }
        }
        Message::PickMountPoint => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();

            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                state.config.contents.mount_point = path;

                if state.config.check_mount_point() {
                    state.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                }
            }

            message_queue.push((Message::PairHomebrewOsc, SharedString::new()));
            message_queue.push((Message::RefreshAll, SharedString::new()));
            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::ToggleShowWii => {
            state.config.contents.show_wii = !state.config.contents.show_wii;
            *state.show_wii.borrow_mut() = state.config.contents.show_wii;
            state.filtered_games.reset();

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::ToggleShowGc => {
            state.config.contents.show_gc = !state.config.contents.show_gc;
            *state.show_gc.borrow_mut() = state.config.contents.show_gc;
            state.filtered_games.reset();

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetWiiOutputFormat => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.wii_output_format = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetGcOutputFormat => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.gc_output_format = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetAlwaysSplit => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.always_split = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetScrubUpdatePartition => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.scrub_update_partition = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetRemoveSourcesGames => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.remove_sources_games = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetRemoveSourcesApps => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.remove_sources_apps = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetTxtCodesSource => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.txt_codes_source = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetThemePreference => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.theme_preference = value;

            #[cfg(windows)]
            match value {
                twbm_core::config::ThemePreference::System => {}
                twbm_core::config::ThemePreference::Light => {
                    message_queue.push((Message::SetWindowColor, "false".to_shared_string()));
                }
                twbm_core::config::ThemePreference::Dark => {
                    message_queue.push((Message::SetWindowColor, "true".to_shared_string()));
                }
            }

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetViewAs => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.view_as = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetSortBy => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.sort_by = value;
            *state.sort_by.borrow_mut() = value;
            state.sorted_games.reset();
            state.sorted_homebrew_apps.reset();

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::SetPreferredLanguage => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.preferred_language = value;

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::WiiloadLocalFile => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();
            let wii_ip = args.next().unwrap().to_string();

            if let Some(in_path) = dialogs::pick_wiiload(&window_handle) {
                let text = slint::format!("Sending {} to Wii...", in_path.display());
                state.notifications.push(Notification::info(text));

                state.config.contents.wii_ip = wii_ip.clone();

                let weak = weak.clone();
                std::thread::spawn(move || {
                    let res = twbm_core::wiiload::send(&wii_ip, &in_path);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(text) => {
                                dispatcher
                                    .invoke_dispatch(Message::NotifyInfo, text.to_shared_string());
                            }
                            Err(e) => {
                                let text = slint::format!("Could not send file to Wii: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });
            }

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::WiiloadOscApp => {
            let wii_ip = args.next().unwrap().to_string();
            let slug = args.next().unwrap();

            state.config.contents.wii_ip = wii_ip.clone();

            let app = state
                .osc_apps
                .iter()
                .find(|app| app.slug == slug)
                .unwrap()
                .clone();

            let text = slint::format!("Sending {} to Wii...", &app.name);
            state.notifications.push(Notification::info(text));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = app.wiiload(&wii_ip);

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(text) => {
                            dispatcher.invoke_dispatch(Message::NotifyInfo, text.to_shared_string())
                        }
                        Err(e) => {
                            let text = slint::format!("Could not send file to Wii: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text)
                        }
                    }
                });
            });

            message_queue.push((Message::SyncConfig, SharedString::new()));
        }
        Message::RefreshAll => {
            let app = weak.upgrade().unwrap();

            let root_path = &state.config.contents.mount_point;

            let new_games = games::scan_drive(root_path);
            let new_apps = homebrew_apps::scan_drive(root_path);
            let new_drive_info = DriveInfo::from_path(root_path).unwrap_or(DriveInfo::empty());

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

            state.games = new_games;
            state.homebrew_apps = new_apps;
            state.drive_info = new_drive_info;

            state.displayed_games.set_vec(new_displayed_games);
            state.displayed_homebrew_apps.set_vec(new_displayed_apps);
            app.global::<UiState<'_>>()
                .set_drive_info(new_displayed_drive_info);

            if !state.is_downloading_covers {
                state.is_downloading_covers = true;

                let weak = weak.clone();
                let preferred_language = state.config.contents.preferred_language;

                let _ = std::thread::spawn(move || {
                    let res = covers::download_covers(ids, preferred_language, &weak);

                    if let Err(e) = res {
                        let _ = weak.upgrade_in_event_loop(move |app| {
                            let text = slint::format!("Could not download covers: {e}");
                            app.global::<Dispatcher<'_>>()
                                .invoke_dispatch(Message::NotifyError, text);
                        });
                    }
                });
            }

            message_queue.push((Message::PairHomebrewOsc, SharedString::new()));
        }
        Message::PairHomebrewOsc => {
            let mut displayed_apps = state
                .homebrew_apps
                .iter()
                .map(DisplayedHomebrewApp::from)
                .collect::<Vec<_>>();

            for app in &mut displayed_apps {
                if let Some(osc_app) = state
                    .osc_apps
                    .iter()
                    .find(|osc_app| osc_app.name.as_str() == app.name.as_str())
                {
                    app.osc_app = DisplayedOscApp::from(osc_app);
                }
            }

            state.displayed_homebrew_apps.set_vec(displayed_apps);
        }
        Message::OpenThat => {
            let uri = args.next().unwrap();

            if let Err(e) = open::that(uri) {
                let text = slint::format!("Failed to open URL: {e}");
                state.notifications.push(Notification::error(text));
            }
        }
        Message::DownloadOscIcons => {
            if !state.is_downloading_osc_icons {
                state.is_downloading_osc_icons = true;

                let weak = weak.clone();
                let apps = state.osc_apps.clone();

                let _ = std::thread::spawn(move || {
                    osc::download_icons(&apps, weak);
                });
            }
        }
        Message::CheckForUpdates => {
            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = twbm_core::updates::check();

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(Some(version)) => {
                            let value = slint::format!("v{version}");
                            dispatcher.invoke_dispatch(Message::SetLatestVersion, value);
                        }
                        Ok(None) => {
                            eprintln!("No updates available");
                        }
                        Err(e) => {
                            let text = slint::format!("Failed to check for updates: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                    }
                });
            });
        }
        Message::CacheOscContents => {
            let force_refresh = args.next().unwrap().parse().unwrap();
            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = twbm_core::osc::cache_contents(&DATA_DIR, force_refresh);

                let _ = weak.upgrade_in_event_loop(|app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    if let Err(e) = res {
                        let text = slint::format!("Failed to cache OSC contents: {e}");
                        dispatcher.invoke_dispatch(Message::NotifyError, text);
                    } else {
                        dispatcher.invoke_dispatch(Message::OscContentsCached, SharedString::new());
                    }
                });
            });
        }
        Message::OscContentsCached => {
            let app = weak.upgrade().unwrap();

            let (new, hours, minutes) =
                twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

            let displayed_apps = new.iter().map(DisplayedOscApp::from).collect::<Vec<_>>();

            state.osc_apps = new;

            state.displayed_osc_apps.set_vec(displayed_apps);

            let ui_state = app.global::<UiState<'_>>();
            ui_state.set_osc_refreshed_x_hours_ago(hours);
            ui_state.set_osc_refreshed_x_minutes_ago(minutes);

            message_queue.push((Message::PairHomebrewOsc, SharedString::new()));
        }
        Message::ReloadOscIcon => {
            let i = args.next().unwrap().parse().unwrap();
            let mut app = state.displayed_osc_apps.row_data(i).unwrap();
            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.slug));

            if let Ok(icon) = Image::load_from_path(&icon_path) {
                app.icon = icon;
                state.displayed_osc_apps.set_row_data(i, app);
            }
        }
        Message::FilterGames => {
            let filter = args.next().unwrap();
            *state.games_filter.borrow_mut() = filter.to_string();
            state.filtered_games.reset();
        }
        Message::FilterHomebrewApps => {
            let filter = args.next().unwrap();
            *state.homebrew_apps_filter.borrow_mut() = filter.to_string();
            state.filtered_homebrew_apps.reset();
        }
        Message::FilterOscApps => {
            let filter = args.next().unwrap();
            *state.osc_apps_filter.borrow_mut() = filter.to_string();
            state.filtered_osc_apps.reset();
        }
        Message::CloseNotification => {
            let i = args.next().unwrap().parse().unwrap();
            state.notifications.remove(i);
        }
        Message::Checksum => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap().clone();

            let weak = weak.clone();

            std::thread::spawn(move || {
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = slint::format!("{percentage}%");
                    let _ = weak2.upgrade_in_event_loop(move |app| {
                        app.global::<Dispatcher<'_>>()
                            .invoke_dispatch(Message::SetCrc32Status, status);
                    });
                };

                let res = checksum::perform(game, &update_progress);

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(crc32) => {
                            let status = slint::format!("{crc32:08x}");
                            dispatcher.invoke_dispatch(Message::SetCrc32Status, status);
                        }
                        Err(e) => {
                            let text = slint::format!("Checksum failed: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                    }
                });
            });
        }
        Message::PickGames => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();
            let recursively = args.next().unwrap().parse().unwrap();

            let paths = if recursively {
                dialogs::pick_games_r(&window_handle)
            } else {
                dialogs::pick_games(&window_handle)
            };

            let existing_ids = state.games.iter().map(|g| g.id).collect::<Vec<_>>();

            state.games_to_add.clear();
            for path in paths {
                if let Ok(mut f) = File::open(&path)
                    && let Ok(meta) = wii_disc_info::Meta::read(&mut f)
                    && let Some(game_id) = GameID::new(meta.game_id())
                    && existing_ids.iter().all(|id| *id != game_id)
                {
                    state
                        .games_to_add
                        .push(path.to_string_lossy().to_shared_string());
                }
            }
        }
        Message::ConfirmGamesToAdd => {
            for path in state.games_to_add.iter() {
                let conv = QueuedConversion::Standard(PathBuf::from(&path));
                let displayed_conv = conv.to_shared_string();
                state.conversion_queue.push(conv);
                state.displayed_conversion_queue.push(displayed_conv);
            }
            state.games_to_add.clear();

            if !state.is_converting {
                state.is_converting = true;
                message_queue.push((Message::TriggerConversion, SharedString::new()));
            }
        }
        Message::TriggerConversion => {
            let Some(conv) = state.conversion_queue.pop_front() else {
                state.is_converting = false;
                let text = "Conversion queue empty";
                state.notifications.push(Notification::info(text));
                return;
            };

            let _ = state.displayed_conversion_queue.remove(0);

            let weak = weak.clone();
            let drive_info = state.drive_info.clone();
            let config = state.config.clone();

            let _ = std::thread::spawn(move || {
                perform_conversion(conv, config, drive_info, weak);
            });
        }
        Message::ClearGamesToAdd => {
            state.games_to_add.clear();
        }
        Message::SetCrc32Status => {
            let app = weak.upgrade().unwrap();
            let status = args.next().unwrap();

            app.global::<UiState<'_>>()
                .set_crc32_status(status.to_shared_string());
        }
        Message::ScrubGame => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap().clone();

            let conv = QueuedConversion::Scrub(game);
            let displayed_conv = conv.to_shared_string();

            state.conversion_queue.push(conv);
            state.displayed_conversion_queue.push(displayed_conv);

            if !state.is_converting {
                state.is_converting = true;
                message_queue.push((Message::TriggerConversion, SharedString::new()));
            }
        }
        Message::PickHomebrewApps => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();
            let paths = dialogs::pick_homebrew_apps(&window_handle);

            let res = twbm_core::util::install_zips(&state.config.contents.mount_point, &paths);

            if let Err(e) = res {
                let text = slint::format!("Failed to install apps: {e}");
                state.notifications.push(Notification::error(text));
            } else {
                let text = slint::format!("{} apps installed successfully", paths.len());
                state.notifications.push(Notification::info(text));
            }

            message_queue.push((Message::RefreshAll, SharedString::new()));
        }
        Message::InstallOscApp => {
            let slug = args.next().unwrap();
            let osc_app_meta = state
                .osc_apps
                .iter()
                .find(|app| app.slug == slug)
                .unwrap()
                .clone();

            let root_dir = state.config.contents.mount_point.clone();

            let text = slint::format!("Installing {}", &osc_app_meta.name);
            state.notifications.push(Notification::info(text));

            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = osc_app_meta.install(&root_dir);

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    if let Err(e) = res {
                        dispatcher.invoke_dispatch(Message::NotifyError, e.to_shared_string());
                    } else {
                        let text = slint::format!("{} installed successfully", &osc_app_meta.name);
                        dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                    }

                    dispatcher.invoke_dispatch(Message::RefreshAll, SharedString::new());
                });
            });
        }
        Message::ReloadCover => {
            let i = args.next().unwrap().parse().unwrap();
            let mut game = state.displayed_games.row_data(i).unwrap();
            let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

            if let Ok(cover) = Image::load_from_path(&cover_path) {
                game.cover = cover;
                state.displayed_games.set_row_data(i, game);
            }
        }
        Message::FinishedDownloadingCovers => {
            state.is_downloading_covers = false;
        }
        Message::DeleteGame => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            if let Err(e) = fs::remove_dir_all(&game.path) {
                let text = slint::format!("Failed to delete game: {e}");
                state.notifications.push(Notification::error(text));
            }

            message_queue.push((Message::RefreshAll, SharedString::new()));
        }
        Message::DeleteHomebrewApp => {
            let path = Path::new(args.next().unwrap());
            let app = state
                .homebrew_apps
                .iter()
                .find(|app| app.path == path)
                .unwrap();

            if let Err(e) = fs::remove_dir_all(&app.path) {
                let text = slint::format!("Failed to delete homebrew app: {e}");
                state.notifications.push(Notification::error(text));
            }

            message_queue.push((Message::RefreshAll, SharedString::new()));
        }
        Message::ScrubAllGames => {
            let to_scrub = state
                .games
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

                    worth.then_some(game.path.to_string_lossy().to_shared_string())
                })
                .collect::<Vec<_>>();

            if to_scrub.is_empty() {
                let text = "No games need scrubbing";
                state.notifications.push(Notification::info(text));
            }

            for path in to_scrub {
                message_queue.push((Message::ScrubGame, path));
            }
        }
        Message::NormalizeDirLayout => {
            match normalize_dir_layout::perform(&state.config.contents.mount_point) {
                Ok(_) => {
                    let text = "Directory layout successfully normalized";
                    state.notifications.push(Notification::info(text));
                }
                Err(e) => {
                    let text = slint::format!("Failed to normalize directory layout: {e}");
                    state.notifications.push(Notification::error(text));
                }
            }
        }
        Message::CancelConversion => {
            let i = args.next().unwrap().parse().unwrap();
            let _ = state.conversion_queue.remove(i);
            let _ = state.displayed_conversion_queue.remove(i);
        }
        Message::CancelAllConversions => {
            state.conversion_queue.clear();
            state.displayed_conversion_queue.clear();
        }
        Message::DownloadTxtCodes => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();
            let game_id = game.id;

            let config = state.config.clone();

            let text = slint::format!("Downloading txtcodes for {game_id}");
            state.notifications.push(Notification::info(text));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = twbm_core::txtcodes::download_cheats(game_id, &config);

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(_) => {
                            let text = slint::format!("Downloaded txtcodes for {game_id}");
                            dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                        }
                        Err(e) => {
                            let text =
                                slint::format!("Failed to download txtcodes for {game_id}: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                    }
                });
            });
        }
        Message::DownloadAllCovers => {
            let for_wiiflow: bool = args.next().unwrap().parse().unwrap();
            let config = state.config.clone();

            let ids = state.games.iter().map(|g| g.id).collect::<Vec<_>>();

            let text = if for_wiiflow {
                "Downloading covers for WiiFlow..."
            } else {
                "Downloading covers for USBLoaderGX..."
            };

            state.notifications.push(Notification::info(text));

            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                let res = if for_wiiflow {
                    twbm_core::covers::download_all_covers_for_wiiflow(&ids, &config)
                } else {
                    twbm_core::covers::download_all_covers_for_usbloadergx(&ids, &config)
                };

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(failed_ids) if failed_ids.is_empty() => {
                            let text = "All covers downloaded successfully".to_shared_string();
                            dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                        }
                        Ok(failed_ids) => {
                            let failed_ids = twbm_core::game_id::make_list_string(&failed_ids);
                            let text = slint::format!(
                                "Covers downloaded successfully\nThe following games may lack some covers: {failed_ids}"
                            );
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                        Err(e) => {
                            let text = slint::format!("Failed to download covers: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                    }
                });
            });
        }
        Message::DownloadAllBanners => {
            let mount_point = state.config.contents.mount_point.clone();

            let ids = state
                .games
                .iter()
                .filter(|g| !g.is_wii)
                .map(|g| g.id)
                .collect::<Vec<_>>();

            let text = slint::format!("Downloading banners for {} games", ids.len());
            state.notifications.push(Notification::info(text));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = twbm_core::banners::download_banners(&mount_point, &ids);

                let _ = weak.upgrade_in_event_loop(move |app| {
                    let dispatcher = app.global::<Dispatcher<'_>>();

                    match res {
                        Ok(failed_ids) if failed_ids.is_empty() => {
                            let text = "All banners downloaded successfully".to_shared_string();
                            dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                        }
                        Ok(failed_ids) => {
                            let failed_ids = twbm_core::game_id::make_list_string(&failed_ids);
                            let text = slint::format!(
                                "Banners downloaded successfully\nExcept the following: {failed_ids}"
                            );
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                        Err(e) => {
                            let text = slint::format!("Failed to download banners: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        }
                    }
                });
            });
        }
        Message::ArchiveManually => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();

            let Some(in_path) = dialogs::pick_game(&window_handle) else {
                return;
            };

            let Some(stem) = in_path.file_stem().and_then(OsStr::to_str) else {
                return;
            };

            let Some(out_path) = dialogs::save_game(&window_handle, stem) else {
                return;
            };

            let conv = QueuedConversion::Archive(in_path, out_path);
            let displayed_conv = conv.to_shared_string();

            state.conversion_queue.push(conv);
            state.displayed_conversion_queue.push(displayed_conv);

            if !state.is_converting {
                state.is_converting = true;
                message_queue.push((Message::TriggerConversion, SharedString::new()));
            }
        }
        Message::SetLatestVersion => {
            let app = weak.upgrade().unwrap();
            let version = args.next().unwrap();

            app.global::<UiState<'_>>()
                .set_latest_version(version.into());
        }
        Message::LoadGameInfo => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            if let Some(disc_path) = game.get_disc_path()
                && let Some(info) = DiscInfo::from_path(disc_path)
            {
                let app = weak.upgrade().unwrap();
                let info = DisplayedDiscInfo::from(&info);
                app.global::<UiState<'_>>().set_current_disc_info(info);
            }
        }
        Message::ArchiveGame => {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            let Some(in_path) = game.get_disc_path() else {
                let text = "No disc found for this game!";
                state.notifications.push(Notification::error(text));

                return;
            };

            let out_path = dialogs::save_game(&window_handle, &game.title);

            if let Some(out_path) = out_path {
                let conv = QueuedConversion::Archive(in_path, out_path);
                let displayed_conv = conv.to_shared_string();

                state.conversion_queue.push(conv);
                state.displayed_conversion_queue.push(displayed_conv);

                if !state.is_converting {
                    state.is_converting = true;
                    message_queue.push((Message::TriggerConversion, SharedString::new()));
                }
            }
        }
        Message::CheckMountPoint => {
            if state.config.check_mount_point() {
                state.notifications.push(Notification::info(NEW_DRIVE_TEXT));
            }
        }
        Message::SetStatus => {
            let app = weak.upgrade().unwrap();
            let status = args.next().unwrap();

            app.global::<UiState<'_>>()
                .set_status(status.to_shared_string());
        }
        #[cfg(windows)]
        Message::SetWindowColor => {
            let is_dark = args.next().unwrap().parse().unwrap();
            crate::window_color::set(is_dark);
        }
        #[cfg(not(windows))]
        Message::SetWindowColor => {}
        #[cfg(target_os = "macos")]
        Message::RunDotClean => {
            let res = {
                let root_path = &state.config.contents.mount_point;
                twbm_core::util::run_dot_clean(root_path)
            };

            match res {
                Ok(_) => {
                    let text = "Successfully ran dot_clean";
                    state.notifications.push(Notification::info(text));
                }
                Err(e) => {
                    let text = slint::format!("Failed to run dot_clean: {e}");
                    state.notifications.push(Notification::error(text));
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        Message::RunDotClean => {}
    }
}
