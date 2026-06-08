// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, Dispatcher, DisplayedConfig, DisplayedDiscInfo, DisplayedDriveInfo, DisplayedGame,
    DisplayedHomebrewApp, DisplayedOscApp, Message, Notification, Page, UiState,
    convert::perform_conversion, covers, dialogs, games, homebrew_apps, osc, state::State, util,
};
use slint::{ComponentHandle, SharedString, ToSharedString, Weak};
use std::{
    collections::VecDeque,
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
    normalize_dir_layout,
};

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking TWBM update has been installed), a path normalization run is recommended\nYou can find it in the Toolbox page";

impl State {
    pub fn update(
        &mut self,
        message: Message,
        payload: SharedString,
        message_queue: &mut VecDeque<(Message, SharedString)>,
        weak: &Weak<AppWindow>,
    ) {
        match message {
            Message::NotifyInfo => {
                self.notifications.push(Notification::info(payload));
            }
            Message::NotifyError => {
                self.notifications.push(Notification::error(payload));
            }
            Message::SyncConfig => {
                let app = weak.upgrade().unwrap();

                app.global::<UiState<'_>>()
                    .set_config(DisplayedConfig::from(&self.config));

                if let Err(e) = self.config.write() {
                    let text = slint::format!("Failed to write config: {e}");
                    self.notifications.push(Notification::error(text));
                }
            }
            Message::PickMountPoint => {
                let app = weak.upgrade().unwrap();
                let window_handle = app.window().window_handle();

                if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                    self.config.contents.mount_point = path;

                    if self.config.check_mount_point() {
                        self.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                    }
                }
                message_queue.push_back((Message::SyncConfig, SharedString::new()));
                message_queue.push_back((Message::RefreshAll, SharedString::new()));
            }
            Message::RefreshDisplayedGames => {
                let displayed_games = self
                    .games
                    .iter()
                    .filter(|game| {
                        ((self.config.contents.show_wii && game.is_wii)
                            || (self.config.contents.show_gc && !game.is_wii))
                            && (self.games_filter.is_empty()
                                || game.search_term.contains(&self.games_filter))
                    })
                    .map(DisplayedGame::from)
                    .collect::<Vec<_>>();

                self.displayed_games.set_vec(displayed_games);
            }
            Message::RefreshDisplayedHomebrewApps => {
                let displayed_homebrew_apps = self
                    .homebrew_apps
                    .iter()
                    .filter(|app| {
                        self.homebrew_apps_filter.is_empty()
                            || app.search_term.contains(&self.homebrew_apps_filter)
                    })
                    .map(|app| {
                        let osc_app = self
                            .osc_apps
                            .iter()
                            .find(|osc_app| osc_app.meta.name.as_str() == app.meta.name.as_str())
                            .map(DisplayedOscApp::from)
                            .unwrap_or_default();

                        DisplayedHomebrewApp::new(app, osc_app)
                    })
                    .collect::<Vec<_>>();

                self.displayed_homebrew_apps
                    .set_vec(displayed_homebrew_apps);
            }
            Message::RefreshDisplayedOscApps => {
                let displayed_osc_apps = self
                    .osc_apps
                    .iter()
                    .filter(|app| {
                        self.osc_apps_filter.is_empty()
                            || app.search_term.contains(&self.osc_apps_filter)
                    })
                    .map(DisplayedOscApp::from)
                    .collect::<Vec<_>>();

                self.displayed_osc_apps.set_vec(displayed_osc_apps);
            }
            Message::ToggleShowWii => {
                self.config.contents.show_wii = !self.config.contents.show_wii;

                message_queue.push_back((Message::RefreshDisplayedGames, SharedString::new()));
                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::ToggleShowGc => {
                self.config.contents.show_gc = !self.config.contents.show_gc;

                message_queue.push_back((Message::RefreshDisplayedGames, SharedString::new()));
                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetWiiOutputFormat => {
                let value = payload.parse().unwrap();
                self.config.contents.wii_output_format = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetGcOutputFormat => {
                let value = payload.parse().unwrap();
                self.config.contents.gc_output_format = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetAlwaysSplit => {
                let value = payload.parse().unwrap();
                self.config.contents.always_split = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetScrubUpdatePartition => {
                let value = payload.parse().unwrap();
                self.config.contents.scrub_update_partition = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetRemoveSourcesGames => {
                let value = payload.parse().unwrap();
                self.config.contents.remove_sources_games = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetRemoveSourcesApps => {
                let value = payload.parse().unwrap();
                self.config.contents.remove_sources_apps = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetTxtCodesSource => {
                let value = payload.parse().unwrap();
                self.config.contents.txt_codes_source = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetThemePreference => {
                let value = payload.parse().unwrap();
                self.config.contents.theme_preference = value;

                #[cfg(windows)]
                if value == twbm_core::config::ThemePreference::Light {
                    crate::window_color::set(false);
                } else if value == twbm_core::config::ThemePreference::Dark {
                    crate::window_color::set(true);
                }

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::SetViewAs => {
                let value = payload.parse().unwrap();
                self.config.contents.view_as = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::RefreshSorting => {
                let compare_games = twbm_core::game::get_compare_fn(self.config.contents.sort_by);
                self.games.sort_by(compare_games);

                let compare_homebrew_apps =
                    twbm_core::homebrew_app::get_compare_fn(self.config.contents.sort_by);
                self.homebrew_apps.sort_by(compare_homebrew_apps);

                message_queue.push_back((Message::RefreshDisplayedGames, SharedString::new()));
                message_queue
                    .push_back((Message::RefreshDisplayedHomebrewApps, SharedString::new()));
            }
            Message::SetSortBy => {
                let value = payload.parse().unwrap();
                self.config.contents.sort_by = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
                message_queue.push_back((Message::RefreshSorting, SharedString::new()));
            }
            Message::SetPreferredLanguage => {
                let value = payload.parse().unwrap();
                self.config.contents.preferred_language = value;

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::WiiloadLocalFile => {
                let app = weak.upgrade().unwrap();
                let window_handle = app.window().window_handle();

                if let Some(in_path) = dialogs::pick_wiiload(&window_handle) {
                    let text = slint::format!("Sending {} to Wii...", in_path.display());
                    self.notifications.push(Notification::info(text));

                    self.config.contents.wii_ip = payload.to_string();

                    let wii_ip = payload.to_string();
                    let weak = weak.clone();
                    std::thread::spawn(move || {
                        let res = twbm_core::wiiload::send(&wii_ip, &in_path);

                        let _ = weak.upgrade_in_event_loop(move |app| {
                            let dispatcher = app.global::<Dispatcher<'_>>();

                            match res {
                                Ok(text) => {
                                    dispatcher.invoke_dispatch(
                                        Message::NotifyInfo,
                                        text.to_shared_string(),
                                    );
                                }
                                Err(e) => {
                                    let text = slint::format!("Could not send file to Wii: {e}");
                                    dispatcher.invoke_dispatch(Message::NotifyError, text);
                                }
                            }
                        });
                    });
                }

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::WiiloadOscApp => {
                let (wii_ip, slug) = payload.split_once(' ').unwrap();

                self.config.contents.wii_ip = wii_ip.to_string();

                let app = self
                    .osc_apps
                    .iter()
                    .find(|app| app.meta.slug == slug)
                    .unwrap()
                    .clone();

                let text = slint::format!("Sending {} to Wii...", &app.meta.name);
                self.notifications.push(Notification::info(text));

                let wii_ip = wii_ip.to_string();
                let weak = weak.clone();
                std::thread::spawn(move || {
                    let res = app.wiiload(&wii_ip);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(text) => dispatcher
                                .invoke_dispatch(Message::NotifyInfo, text.to_shared_string()),
                            Err(e) => {
                                let text = slint::format!("Could not send file to Wii: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });

                message_queue.push_back((Message::SyncConfig, SharedString::new()));
            }
            Message::RefreshAll => {
                let app = weak.upgrade().unwrap();

                let root_path = &self.config.contents.mount_point;

                self.games = games::scan_drive(root_path);
                self.homebrew_apps = homebrew_apps::scan_drive(root_path);
                self.drive_info = DriveInfo::from_path(root_path).unwrap_or_default();

                let new_displayed_drive_info = DisplayedDriveInfo::from(&self.drive_info);

                app.global::<UiState<'_>>()
                    .set_drive_info(new_displayed_drive_info);

                message_queue.push_back((Message::RefreshSorting, SharedString::new()));
                message_queue.push_back((Message::DownloadCovers, SharedString::new()));
            }
            Message::DownloadCovers => {
                if !self.is_downloading_covers {
                    self.is_downloading_covers = true;

                    let ids = self.games.iter().map(|g| g.id).collect::<Vec<_>>();

                    let weak = weak.clone();
                    let preferred_language = self.config.contents.preferred_language;

                    let _ = std::thread::spawn(move || {
                        let res = covers::download_covers(ids, preferred_language, &weak);

                        let _ = weak.upgrade_in_event_loop(move |app| {
                            let dispatcher = app.global::<Dispatcher<'_>>();

                            if let Err(e) = res {
                                let text = slint::format!("Could not download covers: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }

                            dispatcher.invoke_dispatch(
                                Message::FinishedDownloadingCovers,
                                SharedString::new(),
                            );
                        });
                    });
                }
            }
            Message::FinishedDownloadingCovers => {
                self.is_downloading_covers = false;
            }
            Message::OpenThat => {
                if let Err(e) = open::that(&payload) {
                    let text = slint::format!("Failed to open URL: {e}");
                    self.notifications.push(Notification::error(text));
                }
            }
            Message::DownloadOscIcons => {
                if !self.is_downloading_osc_icons {
                    self.is_downloading_osc_icons = true;

                    let weak = weak.clone();
                    let apps = self.osc_apps.clone();

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
                let force_refresh = payload.parse().unwrap();
                let weak = weak.clone();

                std::thread::spawn(move || {
                    let res = twbm_core::osc::cache_contents(&DATA_DIR, force_refresh);

                    let _ = weak.upgrade_in_event_loop(|app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        if let Err(e) = res {
                            let text = slint::format!("Failed to cache OSC contents: {e}");
                            dispatcher.invoke_dispatch(Message::NotifyError, text);
                        } else {
                            dispatcher
                                .invoke_dispatch(Message::OscContentsCached, SharedString::new());
                        }
                    });
                });
            }
            Message::OscContentsCached => {
                let app = weak.upgrade().unwrap();

                let (new, hours, minutes) =
                    twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

                self.osc_apps = new;

                let ui_self = app.global::<UiState<'_>>();
                ui_self.set_osc_refreshed_x_hours_ago(hours);
                ui_self.set_osc_refreshed_x_minutes_ago(minutes);

                message_queue.push_back((Message::RefreshDisplayedOscApps, SharedString::new()));
                message_queue
                    .push_back((Message::RefreshDisplayedHomebrewApps, SharedString::new()));
            }
            Message::FilterGames => {
                self.games_filter = payload.to_lowercase();
                message_queue.push_back((Message::RefreshDisplayedGames, SharedString::new()));
            }
            Message::FilterHomebrewApps => {
                self.homebrew_apps_filter = payload.to_lowercase();
                message_queue
                    .push_back((Message::RefreshDisplayedHomebrewApps, SharedString::new()));
            }
            Message::FilterOscApps => {
                self.osc_apps_filter = payload.to_lowercase();
                message_queue.push_back((Message::RefreshDisplayedOscApps, SharedString::new()));
            }
            Message::CloseNotification => {
                let i = payload.parse().unwrap();
                self.notifications.remove(i);
            }
            Message::Checksum => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap().clone();

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
                let recursively = payload.parse().unwrap();

                let paths = if recursively {
                    dialogs::pick_games_r(&window_handle)
                } else {
                    dialogs::pick_games(&window_handle)
                };

                let existing_ids = self.games.iter().map(|g| g.id).collect::<Vec<_>>();

                self.games_to_add = paths
                    .into_iter()
                    .filter_map(|path| util::should_add_game(path, &existing_ids))
                    .collect();

                let displayed_games_to_add = self
                    .games_to_add
                    .iter()
                    .map(|p| match p.file_name() {
                        Some(filename) => filename.to_string_lossy().to_shared_string(),
                        None => "?".to_shared_string(),
                    })
                    .collect::<Vec<_>>();

                self.displayed_games_to_add.set_vec(displayed_games_to_add);
            }
            Message::ConfirmGamesToAdd => {
                while let Some(path) = self.games_to_add.pop_front() {
                    let _ = self.displayed_games_to_add.remove(0);

                    let conv = QueuedConversion::Standard(path);
                    let displayed_conv = conv.to_shared_string();
                    self.conversion_queue.push_back(conv);
                    self.displayed_conversion_queue.push(displayed_conv);
                }

                if !self.is_converting {
                    self.is_converting = true;
                    message_queue.push_back((Message::TriggerConversion, SharedString::new()));
                }
            }
            Message::TriggerConversion => {
                let Some(conv) = self.conversion_queue.pop_front() else {
                    self.is_converting = false;
                    let text = "Conversion queue empty";
                    self.notifications.push(Notification::info(text));
                    return;
                };

                let _ = self.displayed_conversion_queue.remove(0);

                let weak = weak.clone();
                let drive_info = self.drive_info.clone();
                let config = self.config.clone();

                let _ = std::thread::spawn(move || {
                    perform_conversion(conv, config, drive_info, weak);
                });
            }
            Message::ClearGamesToAdd => {
                self.games_to_add.clear();
                self.displayed_games_to_add.clear();
            }
            Message::SetCrc32Status => {
                let app = weak.upgrade().unwrap();
                app.global::<UiState<'_>>().set_crc32_status(payload);
            }
            Message::ScrubGame => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap().clone();

                let conv = QueuedConversion::Scrub(game);
                let displayed_conv = conv.to_shared_string();

                self.conversion_queue.push_back(conv);
                self.displayed_conversion_queue.push(displayed_conv);

                if !self.is_converting {
                    self.is_converting = true;
                    message_queue.push_back((Message::TriggerConversion, SharedString::new()));
                }
            }
            Message::PickHomebrewApps => {
                let app = weak.upgrade().unwrap();
                let window_handle = app.window().window_handle();
                let paths = dialogs::pick_homebrew_apps(&window_handle);

                let res = twbm_core::util::install_zips(&self.config.contents.mount_point, &paths);

                if let Err(e) = res {
                    let text = slint::format!("Failed to install apps: {e}");
                    self.notifications.push(Notification::error(text));
                } else {
                    let text = slint::format!("{} apps installed successfully", paths.len());
                    self.notifications.push(Notification::info(text));
                }

                message_queue.push_back((Message::RefreshAll, SharedString::new()));
            }
            Message::InstallOscApp => {
                let slug = payload.as_str();

                let osc_app = self
                    .osc_apps
                    .iter()
                    .find(|app| app.meta.slug == slug)
                    .unwrap()
                    .clone();

                let root_dir = self.config.contents.mount_point.clone();

                let text = slint::format!("Installing {}", &osc_app.meta.name);
                self.notifications.push(Notification::info(text));

                let weak = weak.clone();

                std::thread::spawn(move || {
                    let res = osc_app.install(&root_dir);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        if let Err(e) = res {
                            dispatcher.invoke_dispatch(Message::NotifyError, e.to_shared_string());
                        } else {
                            let text =
                                slint::format!("{} installed successfully", &osc_app.meta.name);
                            dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                        }

                        dispatcher.invoke_dispatch(Message::RefreshAll, SharedString::new());
                    });
                });
            }
            Message::DeleteGame => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap();

                if let Err(e) = fs::remove_dir_all(&game.path) {
                    let text = slint::format!("Failed to delete game: {e}");
                    self.notifications.push(Notification::error(text));
                }

                message_queue.push_back((Message::RefreshAll, SharedString::new()));
            }
            Message::DeleteHomebrewApp => {
                let path = Path::new(&payload);
                let app = self
                    .homebrew_apps
                    .iter()
                    .find(|app| app.path == path)
                    .unwrap();

                if let Err(e) = fs::remove_dir_all(&app.path) {
                    let text = slint::format!("Failed to delete homebrew app: {e}");
                    self.notifications.push(Notification::error(text));
                }

                message_queue.push_back((Message::RefreshAll, SharedString::new()));
            }
            Message::ScrubAllGames => {
                let to_scrub = self
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
                    self.notifications.push(Notification::info(text));
                }

                for path in to_scrub {
                    message_queue.push_back((Message::ScrubGame, path));
                }
            }
            Message::NormalizeDirLayout => {
                match normalize_dir_layout::perform(&self.config.contents.mount_point) {
                    Ok(()) => {
                        let text = "Directory layout successfully normalized";
                        self.notifications.push(Notification::info(text));
                    }
                    Err(e) => {
                        let text = slint::format!("Failed to normalize directory layout: {e}");
                        self.notifications.push(Notification::error(text));
                    }
                }
            }
            Message::CancelConversion => {
                let i = payload.parse().unwrap();
                let _ = self.conversion_queue.remove(i);
                let _ = self.displayed_conversion_queue.remove(i);
            }
            Message::CancelAllConversions => {
                self.conversion_queue.clear();
                self.displayed_conversion_queue.clear();
            }
            Message::DownloadTxtCodes => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap();
                let game_id = game.id;

                let config = self.config.clone();

                let text = slint::format!("Downloading txtcodes for {game_id}");
                self.notifications.push(Notification::info(text));

                let weak = weak.clone();
                std::thread::spawn(move || {
                    let res = twbm_core::txtcodes::download_cheats(game_id, &config);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(()) => {
                                let text = slint::format!("Downloaded txtcodes for {game_id}");
                                dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                            }
                            Err(e) => {
                                let text = slint::format!(
                                    "Failed to download txtcodes for {game_id}: {e}"
                                );
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });
            }
            Message::DownloadAllCoversForUsbLoaderGX => {
                let config = self.config.clone();

                let ids = self.games.iter().map(|g| g.id).collect::<Vec<_>>();

                let text = "Downloading covers for USBLoaderGX...";
                self.notifications.push(Notification::info(text));

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    let res = twbm_core::covers::download_all_covers_for_usbloadergx(&ids, &config);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(failed_ids) => {
                                let payload = util::display_list(&failed_ids);
                                dispatcher.invoke_dispatch(
                                    Message::FinishedDownloadingAllCovers,
                                    payload,
                                );
                            }
                            Err(e) => {
                                let text = slint::format!("Failed to download covers: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });
            }
            Message::DownloadAllCoversForWiiFlow => {
                let config = self.config.clone();

                let ids = self.games.iter().map(|g| g.id).collect::<Vec<_>>();

                let text = "Downloading covers for WiiFlow...";
                self.notifications.push(Notification::info(text));

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    let res = twbm_core::covers::download_all_covers_for_wiiflow(&ids, &config);

                    let _ = weak.upgrade_in_event_loop(move |app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(failed_ids) => {
                                let payload = util::display_list(&failed_ids);
                                dispatcher.invoke_dispatch(
                                    Message::FinishedDownloadingAllCovers,
                                    payload,
                                );
                            }
                            Err(e) => {
                                let text = slint::format!("Failed to download covers: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });
            }
            Message::FinishedDownloadingAllCovers => {
                let text = if payload.is_empty() {
                    "All covers downloaded successfully".into()
                } else {
                    slint::format!(
                        "Covers downloaded successfully\nThe following games may lack some covers: {payload}"
                    )
                };

                self.notifications.push(Notification::info(text));
            }
            Message::DownloadAllBanners => {
                let mount_point = self.config.contents.mount_point.clone();

                let ids = self
                    .games
                    .iter()
                    .filter(|g| !g.is_wii)
                    .map(|g| g.id)
                    .collect::<Vec<_>>();

                let text = slint::format!("Downloading banners for {} games", ids.len());
                self.notifications.push(Notification::info(text));

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
                            let failed_ids = util::display_list(&failed_ids);
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

                self.conversion_queue.push_back(conv);
                self.displayed_conversion_queue.push(displayed_conv);

                if !self.is_converting {
                    self.is_converting = true;
                    message_queue.push_back((Message::TriggerConversion, SharedString::new()));
                }
            }
            Message::SetLatestVersion => {
                let app = weak.upgrade().unwrap();
                app.global::<UiState<'_>>().set_latest_version(payload);
            }
            Message::LoadGameInfo => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap();

                if let Some(disc_path) = game.get_disc_path()
                    && let Some(info) = DiscInfo::from_path(disc_path)
                {
                    let app = weak.upgrade().unwrap();
                    let info = DisplayedDiscInfo::from(&info);
                    app.global::<UiState<'_>>().set_current_disc_info(info);
                }
            }
            Message::ArchiveGame => {
                let path = Path::new(&payload);
                let game = self.games.iter().find(|g| g.path == path).unwrap();

                let Some(in_path) = game.get_disc_path() else {
                    let text = "No disc found for this game!";
                    self.notifications.push(Notification::error(text));

                    return;
                };

                let app = weak.upgrade().unwrap();
                let window_handle = app.window().window_handle();
                let out_path = dialogs::save_game(&window_handle, &game.title);

                if let Some(out_path) = out_path {
                    let conv = QueuedConversion::Archive(in_path, out_path);
                    let displayed_conv = conv.to_shared_string();

                    self.conversion_queue.push_back(conv);
                    self.displayed_conversion_queue.push(displayed_conv);

                    if !self.is_converting {
                        self.is_converting = true;
                        message_queue.push_back((Message::TriggerConversion, SharedString::new()));
                    }
                }
            }
            Message::CheckMountPoint => {
                if self.config.check_mount_point() {
                    self.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                }
            }
            Message::SetStatus => {
                let app = weak.upgrade().unwrap();
                app.global::<UiState<'_>>().set_status(payload);
            }
            Message::DownloadWiitdbXml => {
                self.notifications
                    .push(Notification::info("Downloading wiitdb.xml..."));

                let mount_point = self.config.contents.mount_point.clone();
                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    let res = twbm_core::util::download_wiitdb_xml(&mount_point);

                    let _ = weak.upgrade_in_event_loop(|app| {
                        let dispatcher = app.global::<Dispatcher<'_>>();

                        match res {
                            Ok(()) => {
                                let text = "wiitdb.xml downloaded successfully.".into();
                                dispatcher.invoke_dispatch(Message::NotifyInfo, text);
                            }
                            Err(e) => {
                                let text = slint::format!("Failed to download wiitdb.xml: {e}");
                                dispatcher.invoke_dispatch(Message::NotifyError, text);
                            }
                        }
                    });
                });
            }
            Message::FileDropped => {
                let app = weak.upgrade().unwrap();

                if app.global::<UiState<'_>>().get_current_page() == Page::Games {
                    let path = PathBuf::from(&payload);
                    let existing_ids = self.games.iter().map(|g| g.id).collect::<Vec<_>>();

                    if let Some(path) = util::should_add_game(path, &existing_ids)
                        && let Some(filename) = path.file_name()
                    {
                        let displayed_games_to_add =
                            vec![filename.to_string_lossy().to_shared_string()];

                        self.games_to_add = VecDeque::from([path]);
                        self.displayed_games_to_add.set_vec(displayed_games_to_add);
                    }
                }
            }
            #[cfg(windows)]
            Message::SetWindowColorLight => {
                crate::window_color::set(false);
            }
            #[cfg(windows)]
            Message::SetWindowColorDark => {
                crate::window_color::set(true);
            }
            #[cfg(not(windows))]
            Message::SetWindowColorLight | Message::SetWindowColorDark => {}
            #[cfg(target_os = "macos")]
            Message::RunDotClean => {
                let res = {
                    let root_path = &self.config.contents.mount_point;
                    twbm_core::util::run_dot_clean(root_path)
                };

                match res {
                    Ok(()) => {
                        let text = "Successfully ran dot_clean";
                        self.notifications.push(Notification::info(text));
                    }
                    Err(e) => {
                        let text = slint::format!("Failed to run dot_clean: {e}");
                        self.notifications.push(Notification::error(text));
                    }
                }
            }
            #[cfg(not(target_os = "macos"))]
            Message::RunDotClean => {}
        }
    }
}
