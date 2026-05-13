// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::OsStr,
    fs::{self, File},
    path::Path,
};

use crate::{
    Action, ConversionKind, DisplayedConfig, DisplayedDiscInfo, DisplayedDriveInfo, DisplayedGame,
    DisplayedHomebrewApp, DisplayedOscApp, Logic, Notification, QueuedConversion,
    convert::Conversion, covers, dialogs, games, homebrew_apps, osc, state::State,
};
use slint::{Image, Model, SharedString, ToSharedString, Weak};
use smallvec::SmallVec;
use twbm_core::{
    checksum,
    data_dir::DATA_DIR,
    disc_info::{DiscInfo, is_worth_scrubbing},
    drive_info::DriveInfo,
    game_id::GameID,
    normalize_dir_layout,
};

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking TWBM update has been installed), a path normalization run is recommended\nYou can find it in the Toolbox page";

pub fn update<SG, SH, FG, FH, FO>(
    state: &mut State<SG, SH, FG, FH, FO>,
    weak: &Weak<Logic<'static>>,
    window_handle: &slint::WindowHandle,
    action: Action,
    args: SharedString,
    action_queue: &mut SmallVec<(Action, SharedString), 100>,
) where
    SG: FnMut(&DisplayedGame, &DisplayedGame) -> std::cmp::Ordering + 'static,
    SH: FnMut(&DisplayedHomebrewApp, &DisplayedHomebrewApp) -> std::cmp::Ordering + 'static,
    FG: Fn(&DisplayedGame) -> bool + 'static,
    FH: Fn(&DisplayedHomebrewApp) -> bool + 'static,
    FO: Fn(&DisplayedOscApp) -> bool + 'static,
{
    let mut args = args.split('\0');

    match action {
        Action::NotifyInfo => {
            let msg = args.next().unwrap();
            state.notifications.push(Notification::info(msg));
        }
        Action::NotifyError => {
            let msg = args.next().unwrap();
            state.notifications.push(Notification::error(msg));
        }
        Action::SyncConfig => {
            let logic = weak.upgrade().unwrap();

            logic.set_config(DisplayedConfig::from(&state.config));

            if let Err(e) = state.config.write() {
                let msg = slint::format!("Failed to write config: {e}");
                state.notifications.push(Notification::error(msg));
            }
        }
        Action::PickMountPoint => {
            if let Some(path) = dialogs::pick_mount_point(window_handle) {
                state.config.contents.mount_point = path;

                if state.config.check_mount_point() {
                    state.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                }
            }

            action_queue.push((Action::PairHomebrewOsc, SharedString::new()));
            action_queue.push((Action::RefreshAll, SharedString::new()));
            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::ToggleShowWii => {
            state.config.contents.show_wii = !state.config.contents.show_wii;
            *state.show_wii.borrow_mut() = state.config.contents.show_wii;
            state.filtered_games.reset();

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::ToggleShowGc => {
            state.config.contents.show_gc = !state.config.contents.show_gc;
            *state.show_gc.borrow_mut() = state.config.contents.show_gc;
            state.filtered_games.reset();

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetWiiOutputFormat => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.wii_output_format = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetGcOutputFormat => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.gc_output_format = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetAlwaysSplit => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.always_split = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetScrubUpdatePartition => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.scrub_update_partition = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetRemoveSourcesGames => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.remove_sources_games = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetRemoveSourcesApps => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.remove_sources_apps = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetTxtCodesSource => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.txt_codes_source = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetThemePreference => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.theme_preference = value;

            #[cfg(windows)]
            match value {
                twbm_core::config::ThemePreference::System => {}
                twbm_core::config::ThemePreference::Light => {
                    action_queue.push((Action::SetWindowColor, "false".to_shared_string()));
                }
                twbm_core::config::ThemePreference::Dark => {
                    action_queue.push((Action::SetWindowColor, "true".to_shared_string()));
                }
            }

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetViewAs => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.view_as = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetSortBy => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.sort_by = value;
            *state.sort_by.borrow_mut() = value;
            state.sorted_games.reset();
            state.sorted_homebrew_apps.reset();

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::SetPreferredLanguage => {
            let value = args.next().unwrap().parse().unwrap();
            state.config.contents.preferred_language = value;

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::WiiloadLocalFile => {
            let wii_ip = args.next().unwrap().to_string();

            if let Some(in_path) = dialogs::pick_wiiload(window_handle) {
                let msg = slint::format!("Sending {} to Wii...", in_path.display());
                state.notifications.push(Notification::info(msg));

                state.config.contents.wii_ip = wii_ip.clone();

                let weak = weak.clone();
                std::thread::spawn(move || {
                    let res = twbm_core::wiiload::send(&wii_ip, &in_path);

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
            }

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::WiiloadOscApp => {
            let wii_ip = args.next().unwrap().to_string();
            let slug = args.next().unwrap();

            state.config.contents.wii_ip = wii_ip.clone();

            let app = state
                .osc_apps
                .iter()
                .find(|app| app.slug == slug)
                .unwrap()
                .clone();

            let msg = slint::format!("Sending {} to Wii...", &app.name);
            state.notifications.push(Notification::info(msg));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = app.wiiload(&wii_ip);

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(msg) => logic.invoke_dispatch(Action::NotifyInfo, msg.to_shared_string()),
                    Err(e) => {
                        let msg = slint::format!("Could not send file to Wii: {e}");
                        logic.invoke_dispatch(Action::NotifyError, msg)
                    }
                });
            });

            action_queue.push((Action::SyncConfig, SharedString::new()));
        }
        Action::RefreshAll => {
            let logic = weak.upgrade().unwrap();

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
            logic.set_drive_info(new_displayed_drive_info);

            if !state.is_downloading_covers {
                state.is_downloading_covers = true;

                let weak = weak.clone();
                let preferred_language = state.config.contents.preferred_language;

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

            action_queue.push((Action::PairHomebrewOsc, SharedString::new()));
        }
        Action::PairHomebrewOsc => {
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
        Action::OpenThat => {
            let uri = args.next().unwrap();

            if let Err(e) = open::that(uri) {
                let msg = slint::format!("Failed to open URL: {e}");
                state.notifications.push(Notification::error(msg));
            }
        }
        Action::DownloadOscIcons => {
            if !state.is_downloading_osc_icons {
                state.is_downloading_osc_icons = true;

                let weak = weak.clone();
                let apps = state.osc_apps.clone();

                let _ = std::thread::spawn(move || {
                    osc::download_icons(&apps, weak);
                });
            }
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
                        logic.invoke_dispatch(Action::OscContentsCached, SharedString::new());
                    }
                });
            });
        }
        Action::OscContentsCached => {
            let logic = weak.upgrade().unwrap();

            let (new, hours, minutes) =
                twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

            let displayed_apps = new.iter().map(DisplayedOscApp::from).collect::<Vec<_>>();

            state.osc_apps = new;

            state.displayed_osc_apps.set_vec(displayed_apps);
            logic.set_osc_refreshed_x_hours_ago(hours);
            logic.set_osc_refreshed_x_minutes_ago(minutes);

            action_queue.push((Action::PairHomebrewOsc, SharedString::new()));
        }
        Action::ReloadOscIcon => {
            let i = args.next().unwrap().parse().unwrap();
            let mut app = state.displayed_osc_apps.row_data(i).unwrap();
            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.slug));

            if let Ok(icon) = Image::load_from_path(&icon_path) {
                app.icon = icon;
                state.displayed_osc_apps.set_row_data(i, app);
            }
        }
        Action::FilterGames => {
            let filter = args.next().unwrap();
            *state.games_filter.borrow_mut() = filter.to_string();
            state.filtered_games.reset();
        }
        Action::FilterHomebrewApps => {
            let filter = args.next().unwrap();
            *state.homebrew_apps_filter.borrow_mut() = filter.to_string();
            state.filtered_homebrew_apps.reset();
        }
        Action::FilterOscApps => {
            let filter = args.next().unwrap();
            *state.osc_apps_filter.borrow_mut() = filter.to_string();
            state.filtered_osc_apps.reset();
        }
        Action::CloseNotification => {
            let i = args.next().unwrap().parse().unwrap();
            state.notifications.remove(i);
        }
        Action::Checksum => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap().clone();

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
        }
        Action::PickGames => {
            let recursively = args.next().unwrap().parse().unwrap();

            let paths = if recursively {
                dialogs::pick_games_r(window_handle)
            } else {
                dialogs::pick_games(window_handle)
            };

            let existing_ids = state.games.iter().map(|g| g.id).collect::<Vec<_>>();

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

            state.conversion_queue_buffer.set_vec(new);
        }
        Action::ConfirmConversionQueueBuffer => {
            state
                .conversion_queue
                .extend(state.conversion_queue_buffer.iter());
            state.conversion_queue_buffer.clear();

            if !state.is_converting {
                state.is_converting = true;
                action_queue.push((Action::TriggerConversion, SharedString::new()));
            }
        }
        Action::TriggerConversion => {
            if state.conversion_queue.row_count() == 0 {
                state.is_converting = false;
                let msg = SharedString::from("Conversion queue empty");
                state.notifications.push(Notification::info(msg));
                return;
            }

            let queued = state.conversion_queue.remove(0);
            let conv = Conversion::new(&queued);

            let weak = weak.clone();
            let drive_info = state.drive_info.clone();
            let config = state.config.clone();

            let _ = std::thread::spawn(move || {
                conv.perform(config, drive_info, weak);
            });
        }
        Action::ClearConversionQueueBuffer => {
            state.conversion_queue_buffer.clear();
        }
        Action::SetCrc32Status => {
            let logic = weak.upgrade().unwrap();
            let status = args.next().unwrap();

            logic.set_crc32_status(status.to_shared_string());
        }
        Action::ScrubGame => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            let Some(disc_path) = game.get_disc_path() else {
                let msg = slint::format!("No disc path found for game {}", game.title);
                state.notifications.push(Notification::error(msg));
                return;
            };

            let conv = QueuedConversion {
                kind: ConversionKind::Scrub,
                in_path: disc_path.to_string_lossy().to_shared_string(),
                game_title: game.title.to_shared_string(),
                game_id: game.id.to_shared_string(),
                ..Default::default()
            };

            state.conversion_queue.push(conv);

            if !state.is_converting {
                state.is_converting = true;
                action_queue.push((Action::TriggerConversion, SharedString::new()));
            }
        }
        Action::PickHomebrewApps => {
            let paths = dialogs::pick_homebrew_apps(window_handle);

            let res = twbm_core::util::install_zips(&state.config.contents.mount_point, &paths);

            if let Err(e) = res {
                let msg = slint::format!("Failed to install apps: {e}");
                state.notifications.push(Notification::error(msg));
            } else {
                let msg = slint::format!("{} apps installed successfully", paths.len());
                state.notifications.push(Notification::info(msg));
            }

            action_queue.push((Action::RefreshAll, SharedString::new()));
        }
        Action::InstallOscApp => {
            let slug = args.next().unwrap();
            let app = state
                .osc_apps
                .iter()
                .find(|app| app.slug == slug)
                .unwrap()
                .clone();

            let root_dir = state.config.contents.mount_point.clone();

            let msg = slint::format!("Installing {}", &app.name);
            state.notifications.push(Notification::info(msg));

            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = app.install(&root_dir);

                let _ = weak.upgrade_in_event_loop(move |logic| {
                    if let Err(e) = res {
                        logic.invoke_dispatch(Action::NotifyError, e.to_shared_string());
                    } else {
                        let msg = slint::format!("{} installed successfully", &app.name);
                        logic.invoke_dispatch(Action::NotifyInfo, msg);
                    }

                    logic.invoke_dispatch(Action::RefreshAll, SharedString::new());
                });
            });
        }
        Action::ReloadCover => {
            let i = args.next().unwrap().parse().unwrap();
            let mut game = state.displayed_games.row_data(i).unwrap();
            let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

            if let Ok(cover) = Image::load_from_path(&cover_path) {
                game.cover = cover;
                state.displayed_games.set_row_data(i, game);
            }
        }
        Action::FinishedDownloadingCovers => {
            state.is_downloading_covers = false;
        }
        Action::DeleteGame => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            if let Err(e) = fs::remove_dir_all(&game.path) {
                let msg = slint::format!("Failed to delete game: {e}");
                state.notifications.push(Notification::error(msg));
            }

            action_queue.push((Action::RefreshAll, SharedString::new()));
        }
        Action::DeleteHomebrewApp => {
            let path = Path::new(args.next().unwrap());
            let app = state
                .homebrew_apps
                .iter()
                .find(|app| app.path == path)
                .unwrap();

            if let Err(e) = fs::remove_dir_all(&app.path) {
                let msg = slint::format!("Failed to delete homebrew app: {e}");
                state.notifications.push(Notification::error(msg));
            }

            action_queue.push((Action::RefreshAll, SharedString::new()));
        }
        Action::ScrubAllGames => {
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
                let msg = "No games need scrubbing";
                state.notifications.push(Notification::info(msg));
            }

            for path in to_scrub {
                action_queue.push((Action::ScrubGame, path));
            }
        }
        Action::NormalizeDirLayout => {
            match normalize_dir_layout::perform(&state.config.contents.mount_point) {
                Ok(_) => {
                    let msg = "Directory layout successfully normalized";
                    state.notifications.push(Notification::info(msg));
                }
                Err(e) => {
                    let msg = slint::format!("Failed to normalize directory layout: {e}");
                    state.notifications.push(Notification::error(msg));
                }
            }
        }
        Action::CancelConversion => {
            let i = args.next().unwrap().parse().unwrap();
            let _ = state.conversion_queue.remove(i);
        }
        Action::CancelAllConversions => {
            state.conversion_queue.clear();
        }
        Action::DownloadTxtCodes => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();
            let game_id = game.id;

            let config = state.config.clone();

            let msg = slint::format!("Downloading txtcodes for {game_id}");
            state.notifications.push(Notification::info(msg));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = twbm_core::txtcodes::download_cheats(game_id, &config);

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(_) => {
                        let msg = slint::format!("Downloaded txtcodes for {game_id}");
                        logic.invoke_dispatch(Action::NotifyInfo, msg);
                    }
                    Err(e) => {
                        let msg = slint::format!("Failed to download txtcodes for {game_id}: {e}");
                        logic.invoke_dispatch(Action::NotifyError, msg);
                    }
                });
            });
        }
        Action::DownloadAllCovers => {
            let for_wiiflow: bool = args.next().unwrap().parse().unwrap();
            let config = state.config.clone();

            let ids = state.games.iter().map(|g| g.id).collect::<Vec<_>>();

            let msg = if for_wiiflow {
                "Downloading covers for WiiFlow..."
            } else {
                "Downloading covers for USBLoaderGX..."
            };

            state.notifications.push(Notification::info(msg));

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
        }
        Action::DownloadAllBanners => {
            let mount_point = state.config.contents.mount_point.clone();

            let ids = state
                .games
                .iter()
                .filter(|g| !g.is_wii)
                .map(|g| g.id)
                .collect::<Vec<_>>();

            let msg = slint::format!("Downloading banners for {} games", ids.len());
            state.notifications.push(Notification::info(msg));

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
        }
        Action::ArchiveManually => {
            let Some(in_path) = dialogs::pick_game(window_handle) else {
                return;
            };

            let Some(stem) = in_path.file_stem().and_then(OsStr::to_str) else {
                return;
            };

            let Some(out_path) = dialogs::save_game(window_handle, stem) else {
                return;
            };

            let queued = QueuedConversion {
                kind: ConversionKind::Archive,
                in_path: in_path.to_string_lossy().to_shared_string(),
                out_path: out_path.to_string_lossy().to_shared_string(),
                ..Default::default()
            };

            state.conversion_queue.push(queued);

            if !state.is_converting {
                state.is_converting = true;
                action_queue.push((Action::TriggerConversion, SharedString::new()));
            }
        }
        Action::SetLatestVersion => {
            let logic = weak.upgrade().unwrap();
            let version = args.next().unwrap();

            logic.set_latest_version(version.into());
        }
        Action::LoadGameInfo => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            if let Some(disc_path) = game.get_disc_path()
                && let Some(info) = DiscInfo::from_path(disc_path)
            {
                let logic = weak.upgrade().unwrap();
                let info = DisplayedDiscInfo::from(&info);
                logic.set_current_disc_info(info);
            }
        }
        Action::ArchiveGame => {
            let path = Path::new(args.next().unwrap());
            let game = state.games.iter().find(|g| g.path == path).unwrap();

            let Some(in_path) = game.get_disc_path() else {
                let msg = "No disc found for this game!";
                state.notifications.push(Notification::error(msg));

                return;
            };

            let out_path = dialogs::save_game(window_handle, &game.title);

            if let Some(out_path) = out_path {
                let queued = QueuedConversion {
                    kind: ConversionKind::Archive,
                    in_path: in_path.to_string_lossy().to_shared_string(),
                    out_path: out_path.to_string_lossy().to_shared_string(),
                    ..Default::default()
                };

                state.conversion_queue.push(queued);

                if !state.is_converting {
                    state.is_converting = true;
                    action_queue.push((Action::TriggerConversion, SharedString::new()));
                }
            }
        }
        Action::CheckMountPoint => {
            if state.config.check_mount_point() {
                state.notifications.push(Notification::info(NEW_DRIVE_TEXT));
            }
        }
        Action::SetStatus => {
            let logic = weak.upgrade().unwrap();
            let status = args.next().unwrap();

            logic.set_status(status.to_shared_string());
        }
        #[cfg(windows)]
        Action::SetWindowColor => {
            let is_dark = args.next().unwrap().parse().unwrap();
            crate::window_color::set(is_dark);
        }
        #[cfg(not(windows))]
        Action::SetWindowColor => {}
        #[cfg(target_os = "macos")]
        Action::RunDotClean => {
            let res = {
                let root_path = &state.config.contents.mount_point;
                twbm_core::util::run_dot_clean(root_path)
            };

            match res {
                Ok(_) => {
                    let msg = "Successfully ran dot_clean";
                    state.notifications.push(Notification::info(msg));
                }
                Err(e) => {
                    let msg = slint::format!("Failed to run dot_clean: {e}");
                    state.notifications.push(Notification::error(msg));
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        Action::RunDotClean => {}
    }
}
