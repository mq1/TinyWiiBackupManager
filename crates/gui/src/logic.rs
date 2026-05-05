// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConversionKind, DisplayedConfig, DisplayedDiscInfo, DisplayedDriveInfo, DisplayedGame,
    DisplayedHomebrewApp, DisplayedOscApp, Logic, Notification, QueuedConversion,
    convert::Conversion, covers, data_dir::DATA_DIR, dialogs, games, homebrew_apps, osc,
};
use slint::{
    FilterModel, Global, Image, Model, ModelRc, SharedString, SortModel, ToSharedString, VecModel,
    Window,
};
use std::{
    cell::RefCell,
    fs::{self, File},
    path::Path,
    rc::Rc,
};
use twbm_core::{
    checksum,
    config::Config,
    disc_info::{DiscInfo, is_worth_scrubbing},
    drive_info::DriveInfo,
    game::Game,
    game_id::GameID,
    homebrew_app::HomebrewApp,
    normalize_dir_layout,
    osc::OscAppMeta,
};

impl Logic<'_> {
    pub fn init(&self, config: Config, window: &Window) {
        let displayed_config = DisplayedConfig::from(&config);
        let config = Rc::new(RefCell::new(config));

        let drive_info = Rc::new(RefCell::new(DriveInfo::empty()));

        let games = Rc::new(RefCell::new(Vec::<Game>::new()));
        let homebrew_apps = Rc::new(RefCell::new(Vec::<HomebrewApp>::new()));
        let osc_apps = Rc::new(RefCell::new(Vec::<OscAppMeta>::new()));

        let displayed_games = Rc::new(VecModel::from(Vec::new()));
        let games_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_games = Rc::new(SortModel::new(
            displayed_games.clone(),
            games::get_compare_fn(config.clone()),
        ));
        let filtered_games = Rc::new(FilterModel::new(
            sorted_games.clone(),
            games::get_filter_fn(games_filter.clone(), config.clone()),
        ));

        let displayed_homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_homebrew_apps = Rc::new(SortModel::new(
            displayed_homebrew_apps.clone(),
            homebrew_apps::get_compare_fn(config.clone()),
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

        let notifications = Rc::new(VecModel::from(Vec::new()));

        let conversion_queue = Rc::new(VecModel::from(Vec::new()));
        let conversion_queue_buffer = Rc::new(VecModel::from(Vec::new()));

        let is_converting = Rc::new(RefCell::new(false));
        let is_downloading_osc_icons = Rc::new(RefCell::new(false));
        let is_downloading_covers = Rc::new(RefCell::new(false));

        self.set_app_version(env!("CARGO_PKG_VERSION").to_shared_string());
        self.set_config(displayed_config);
        self.set_games(ModelRc::from(filtered_games.clone()));
        self.set_homebrew_apps(ModelRc::from(filtered_homebrew_apps.clone()));
        self.set_osc_apps(ModelRc::from(filtered_osc_apps.clone()));
        self.set_notifications(ModelRc::from(notifications.clone()));
        self.set_conversion_queue(ModelRc::from(conversion_queue.clone()));
        self.set_conversion_queue_buffer(ModelRc::from(conversion_queue_buffer.clone()));

        // Mutations

        let config_clone = config.clone();
        let weak = self.as_weak();
        let notifications_clone = notifications.clone();
        self.on_sync_config(move || {
            let logic = weak.upgrade().unwrap();
            let config = config_clone.borrow();
            let displayed_config = DisplayedConfig::from(&*config);

            logic.set_config(displayed_config);
            if let Err(e) = config.write() {
                let msg = format!("Failed to save config: {e}");
                notifications_clone.push(Notification::error(msg));
            }
        });

        let notifications_clone = notifications.clone();
        self.on_open_that(move |uri| {
            if let Err(e) = open::that(uri) {
                let msg = format!("Failed to open URL: {e}");
                notifications_clone.push(Notification::error(msg));
            }
        });

        let weak = self.as_weak();
        let window_handle = window.window_handle();
        let config_clone = config.clone();
        self.on_pick_mount_point(move || {
            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                config_clone.borrow_mut().contents.mount_point = path;

                let logic = weak.upgrade().unwrap();
                logic.invoke_sync_config();
                logic.invoke_refresh_all();
            }
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_wii_output_format(move |format| {
            config_clone.borrow_mut().contents.wii_output_format =
                format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_gc_output_format(move |format| {
            config_clone.borrow_mut().contents.gc_output_format =
                format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_always_split(move |always_split| {
            config_clone.borrow_mut().contents.always_split = always_split;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_scrub_update_partition(move |scrub_update_partition| {
            config_clone.borrow_mut().contents.scrub_update_partition = scrub_update_partition;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_remove_sources_games(move |remove_sources_games| {
            config_clone.borrow_mut().contents.remove_sources_games = remove_sources_games;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_remove_sources_apps(move |remove_sources_apps| {
            config_clone.borrow_mut().contents.remove_sources_apps = remove_sources_apps;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_txt_codes_source(move |source| {
            config_clone.borrow_mut().contents.txt_codes_source =
                source.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_theme_preference(move |theme_preference| {
            config_clone.borrow_mut().contents.theme_preference =
                theme_preference.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_view_as(move |format| {
            config_clone.borrow_mut().contents.view_as = format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let sorted_games_clone = sorted_games.clone();
        let sorted_homebrew_apps_clone = sorted_homebrew_apps.clone();
        let weak = self.as_weak();
        self.on_set_sort_by(move |sort_by| {
            config_clone.borrow_mut().contents.sort_by = sort_by.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();

            sorted_games_clone.reset();
            sorted_homebrew_apps_clone.reset();
        });

        let config_clone = config.clone();
        let filtered_games_clone = filtered_games.clone();
        let weak = self.as_weak();
        self.on_set_show_wii(move |show_wii| {
            config_clone.borrow_mut().contents.show_wii = show_wii;
            weak.upgrade().unwrap().invoke_sync_config();

            filtered_games_clone.reset();
        });

        let config_clone = config.clone();
        let filtered_games_clone = filtered_games.clone();
        let weak = self.as_weak();
        self.on_set_show_gc(move |show_gc| {
            config_clone.borrow_mut().contents.show_gc = show_gc;
            weak.upgrade().unwrap().invoke_sync_config();

            filtered_games_clone.reset();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_preferred_language(move |preferred_language| {
            config_clone.borrow_mut().contents.preferred_language =
                preferred_language.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let window_handle = window.window_handle();
        let weak = self.as_weak();
        let notifications_clone = notifications.clone();
        self.on_wiiload_local_file(move |wii_ip| {
            let in_path = dialogs::pick_wiiload(&window_handle);

            if let Some(in_path) = in_path {
                let msg = slint::format!("Sending {} to Wii...", in_path.display());
                notifications_clone.push(Notification::info(msg));

                config_clone.borrow_mut().contents.wii_ip = wii_ip.to_string();
                weak.upgrade().unwrap().invoke_sync_config();

                let weak = weak.clone();
                std::thread::spawn(move || {
                    let res = twbm_core::wiiload::send(&wii_ip, &in_path);

                    let _ = weak.upgrade_in_event_loop(move |logic| match res {
                        Ok(msg) => logic.invoke_notify_info(msg.to_shared_string()),
                        Err(e) => {
                            let msg = slint::format!("Could not send file to Wii: {e}");
                            logic.invoke_notify_error(msg)
                        }
                    });
                });
            }
        });

        let weak = self.as_weak();
        let config_clone = config.clone();
        let osc_apps_clone = osc_apps.clone();
        let notifications_clone = notifications.clone();
        self.on_wiiload_osc_app(move |wii_ip, i| {
            config_clone.borrow_mut().contents.wii_ip = wii_ip.to_string();
            weak.upgrade().unwrap().invoke_sync_config();

            let app = osc_apps_clone.borrow()[i as usize].clone();

            let msg = slint::format!("Sending {} to Wii...", app.name);
            notifications_clone.push(Notification::info(msg));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = app.wiiload(&wii_ip);

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(msg) => logic.invoke_notify_info(msg.to_shared_string()),
                    Err(e) => {
                        let msg = slint::format!("Could not send file to Wii: {e}");
                        logic.invoke_notify_error(msg)
                    }
                });
            });
        });

        let games_clone = games.clone();
        let homebrew_apps_clone = homebrew_apps.clone();
        let displayed_games_clone = displayed_games.clone();
        let displayed_homebrew_apps_clone = displayed_homebrew_apps.clone();
        let config_clone = config.clone();
        let drive_info_clone = drive_info.clone();
        let weak = self.as_weak();
        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_refresh_all(move || {
            let logic = weak.upgrade().unwrap();

            let (new_games, new_apps, drive_info) = {
                let config = config_clone.borrow();
                let root_path = Path::new(&config.contents.mount_point);

                let p = root_path.to_path_buf();
                let join = std::thread::spawn(move || DriveInfo::from_path(&p));

                let new_games = games::scan_drive(root_path);
                let new_apps = homebrew_apps::scan_drive(root_path);

                let drive_info = join.join().unwrap().unwrap_or(DriveInfo::empty());

                (new_games, new_apps, drive_info)
            };

            let ids = new_games.iter().map(|g| g.id).collect::<Vec<_>>();

            let displayed_games = new_games
                .iter()
                .enumerate()
                .map(|(i, g)| DisplayedGame::new(g, i))
                .collect::<Vec<_>>();

            let displayed_apps = new_apps
                .iter()
                .enumerate()
                .map(|(i, a)| DisplayedHomebrewApp::new(a, i))
                .collect::<Vec<_>>();

            let displayed_drive_info = DisplayedDriveInfo::from(&drive_info);

            *games_clone.borrow_mut() = new_games;
            *homebrew_apps_clone.borrow_mut() = new_apps;
            *drive_info_clone.borrow_mut() = drive_info;

            displayed_games_clone.set_vec(displayed_games);
            displayed_homebrew_apps_clone.set_vec(displayed_apps);
            logic.set_drive_info(displayed_drive_info);

            let mut is_downloading_covers = is_downloading_covers_clone.borrow_mut();
            if !*is_downloading_covers {
                *is_downloading_covers = true;

                let weak = weak.clone();
                let preferred_language = config_clone.borrow().contents.preferred_language;
                let _ = std::thread::spawn(move || {
                    if let Err(e) = covers::download_covers(ids, preferred_language, &weak) {
                        let _ = weak.upgrade_in_event_loop(move |logic| {
                            logic.invoke_notify_error(e.to_shared_string());
                        });
                    }
                });
            }

            logic.invoke_pair_homebrew_osc();
        });

        let weak = self.as_weak();
        self.on_cache_osc_contents(move |force_refresh| {
            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = twbm_core::osc::cache_contents(&DATA_DIR, force_refresh);

                let _ = weak.upgrade_in_event_loop(|logic| {
                    if let Err(e) = res {
                        logic.invoke_notify_error(e.to_shared_string());
                    } else {
                        logic.invoke_osc_contents_cached();
                    }
                });
            });
        });

        let osc_apps_clone = osc_apps.clone();
        let displayed_osc_apps_clone = displayed_osc_apps.clone();
        let weak = self.as_weak();
        self.on_osc_contents_cached(move || {
            let logic = weak.upgrade().unwrap();

            let (new, hours, minutes) =
                twbm_core::osc::load_contents(&DATA_DIR).unwrap_or_default();

            let displayed_apps = new
                .iter()
                .enumerate()
                .map(|(i, a)| DisplayedOscApp::new(a, i))
                .collect::<Vec<_>>();

            *osc_apps_clone.borrow_mut() = new;

            displayed_osc_apps_clone.set_vec(displayed_apps);
            logic.set_osc_refreshed_x_hours_ago(hours);
            logic.set_osc_refreshed_x_minutes_ago(minutes);

            logic.invoke_pair_homebrew_osc();
        });

        let is_downloading_osc_icons_clone = is_downloading_osc_icons.clone();
        let osc_apps_clone = osc_apps.clone();
        let weak = self.as_weak();
        self.on_download_osc_icons(move || {
            let mut is_downloading_osc_icons = is_downloading_osc_icons_clone.borrow_mut();
            if !*is_downloading_osc_icons {
                *is_downloading_osc_icons = true;

                let apps = osc_apps_clone.borrow().clone();

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    osc::download_icons(&apps, weak);
                });
            }
        });

        let displayed_osc_apps_clone = displayed_osc_apps.clone();
        self.on_reload_osc_icon(move |i| {
            let mut app = displayed_osc_apps_clone.row_data(i as usize).unwrap();
            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.slug));

            if let Ok(icon) = Image::load_from_path(&icon_path) {
                app.icon = icon;
                displayed_osc_apps_clone.set_row_data(i as usize, app);
            }
        });

        let games_filter_clone = games_filter.clone();
        let filtered_games_clone = filtered_games.clone();
        self.on_filter_games(move |filter| {
            *games_filter_clone.borrow_mut() = filter;
            filtered_games_clone.reset();
        });

        let homebrew_apps_filter_clone = homebrew_apps_filter.clone();
        let filtered_homebrew_apps_clone = filtered_homebrew_apps.clone();
        self.on_filter_homebrew_apps(move |filter| {
            *homebrew_apps_filter_clone.borrow_mut() = filter;
            filtered_homebrew_apps_clone.reset();
        });

        let osc_apps_filter_clone = osc_apps_filter.clone();
        let filtered_osc_apps_clone = filtered_osc_apps.clone();
        self.on_filter_osc_apps(move |filter| {
            *osc_apps_filter_clone.borrow_mut() = filter;
            filtered_osc_apps_clone.reset();
        });

        let notifications_clone = notifications.clone();
        self.on_close_notification(move |i| {
            notifications_clone.remove(i as usize);
        });

        let games_clone = games.clone();
        let weak = self.as_weak();
        self.on_checksum(move |i| {
            let game = games_clone.borrow()[i as usize].clone();
            let weak = weak.clone();

            let _ = std::thread::spawn(move || {
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = format!("{percentage}%");
                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_crc32_status(status.to_shared_string());
                    });
                };

                let res = checksum::perform(game, &update_progress);

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(crc32) => {
                        logic.set_crc32_status(format!("{crc32:08x}").to_shared_string());
                    }
                    Err(e) => {
                        logic.invoke_notify_error(e.to_shared_string());
                    }
                });
            });
        });

        let notifications_clone = notifications.clone();
        self.on_notify_error(move |e| {
            notifications_clone.push(Notification::error(e));
        });

        let notifications_clone = notifications.clone();
        self.on_notify_info(move |e| {
            notifications_clone.push(Notification::info(e));
        });

        let window_handle = window.window_handle();
        let games_clone = games.clone();
        let conversion_queue_buffer_clone = conversion_queue_buffer.clone();
        self.on_pick_games(move |recursively| {
            let paths = if recursively {
                dialogs::pick_games_r(&window_handle)
            } else {
                dialogs::pick_games(&window_handle)
            };

            let existing_ids = games_clone
                .borrow()
                .iter()
                .map(|g| g.id)
                .collect::<Vec<_>>();

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

            conversion_queue_buffer_clone.set_vec(new);
        });

        let conversion_queue_clone = conversion_queue.clone();
        let conversion_queue_buffer_clone = conversion_queue_buffer.clone();
        let is_converting_clone = is_converting.clone();
        let weak = self.as_weak();
        self.on_confirm_conversion_queue_buffer(move || {
            conversion_queue_clone.extend(conversion_queue_buffer_clone.iter());
            conversion_queue_buffer_clone.clear();

            let mut is_converting = is_converting_clone.borrow_mut();
            if !*is_converting {
                *is_converting = true;
                weak.upgrade().unwrap().invoke_trigger_conversion();
            }
        });

        let config_clone = config.clone();
        let drive_info_clone = drive_info.clone();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_trigger_conversion(move || {
            if conversion_queue_clone.row_count() == 0 {
                *is_converting_clone.borrow_mut() = false;
                let msg = SharedString::from("Conversion queue empty");
                notifications_clone.push(Notification::info(msg));
                return;
            }

            let queued = conversion_queue_clone.remove(0);
            let conv = Conversion::new(&queued);

            let weak = weak.clone();
            let drive_info = *drive_info_clone.borrow();
            let config = config_clone.borrow().clone();

            let _ = std::thread::spawn(move || {
                conv.perform(config, drive_info, weak);
            });
        });

        let conversion_queue_buffer_clone = conversion_queue_buffer.clone();
        self.on_clear_conversion_queue_buffer(move || {
            conversion_queue_buffer_clone.clear();
        });

        let weak = self.as_weak();
        self.on_set_status(move |status| {
            weak.upgrade().unwrap().set_status(status);
        });

        let weak = self.as_weak();
        self.on_set_crc32_status(move |status| {
            weak.upgrade().unwrap().set_crc32_status(status);
        });

        let games_clone = games.clone();
        let window_handle = window.window_handle();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_archive_game(move |i| {
            let (in_path, out_path) = {
                let game = &games_clone.borrow()[i as usize];
                let Some(in_path) = game.get_disc_path() else {
                    let msg = SharedString::from("No disc found for this game!");
                    notifications_clone.push(Notification::error(msg));
                    return;
                };
                let out_path = dialogs::save_game(&window_handle, game);

                (in_path, out_path)
            };

            if let Some(out_path) = out_path {
                let queued = QueuedConversion {
                    kind: ConversionKind::Archive,
                    in_path: in_path.to_string_lossy().to_shared_string(),
                    out_path: out_path.to_string_lossy().to_shared_string(),
                    ..Default::default()
                };

                conversion_queue_clone.push(queued);

                let mut is_converting = is_converting_clone.borrow_mut();
                if !*is_converting {
                    *is_converting = true;
                    weak.upgrade().unwrap().invoke_trigger_conversion();
                }
            }
        });

        let games_clone = games.clone();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_scrub_game(move |i| {
            let conv = {
                let game = &games_clone.borrow()[i as usize];
                let Some(disc_path) = game.get_disc_path() else {
                    let msg = slint::format!("No disc path found for game {}", game.title);
                    notifications_clone.push(Notification::error(msg));
                    return;
                };

                QueuedConversion {
                    kind: ConversionKind::Scrub,
                    in_path: disc_path.to_string_lossy().to_shared_string(),
                    game_title: game.title.to_shared_string(),
                    game_id: game.id.to_shared_string(),
                    ..Default::default()
                }
            };

            conversion_queue_clone.push(conv);

            let mut is_converting = is_converting_clone.borrow_mut();
            if !*is_converting {
                *is_converting = true;
                weak.upgrade().unwrap().invoke_trigger_conversion();
            }
        });

        let config_clone = config.clone();
        let window_handle = window.window_handle();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_pick_homebrew_apps(move || {
            let paths = dialogs::pick_homebrew_apps(&window_handle);

            let res = {
                let config = config_clone.borrow();
                twbm_core::util::install_zips(&config.contents.mount_point, &paths)
            };

            if let Err(e) = res {
                let msg = slint::format!("Failed to install apps: {e}");
                notifications_clone.push(Notification::error(msg));
            } else {
                let msg = slint::format!("{} apps installed successfully", paths.len());
                notifications_clone.push(Notification::info(msg));
                weak.upgrade().unwrap().invoke_refresh_all();
            }
        });

        let osc_apps_clone = osc_apps.clone();
        let config_clone = config.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_install_osc_app(move |i| {
            let app = osc_apps_clone.borrow()[i as usize].clone();
            let root_dir = config_clone.borrow().contents.mount_point.clone();

            let msg = slint::format!("Installing {}", &app.name);
            notifications_clone.push(Notification::info(msg));

            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = app.install(&root_dir);

                let _ = weak.upgrade_in_event_loop(move |logic| {
                    if let Err(e) = res {
                        logic.invoke_notify_error(e.to_shared_string());
                    } else {
                        let msg = slint::format!("{} installed successfully", &app.name);
                        logic.invoke_notify_info(msg);
                        logic.invoke_refresh_all();
                    }
                });
            });
        });

        let displayed_games_clone = displayed_games.clone();
        self.on_reload_cover(move |i| {
            let mut game = displayed_games_clone.row_data(i as usize).unwrap();
            let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

            if let Ok(cover) = Image::load_from_path(&cover_path) {
                game.cover = cover;
                displayed_games_clone.set_row_data(i as usize, game);
            }
        });

        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_finished_downloading_covers(move || {
            *is_downloading_covers_clone.borrow_mut() = false;
        });

        let homebrew_apps_clone = homebrew_apps.clone();
        let osc_apps_clone = osc_apps.clone();
        let displayed_homebrew_apps_clone = displayed_homebrew_apps.clone();
        self.on_pair_homebrew_osc(move || {
            let mut homebrew_apps = homebrew_apps_clone.borrow_mut();
            let osc_apps = osc_apps_clone.borrow();

            for app in homebrew_apps.iter_mut() {
                if let Some(osc_idx) = osc_apps
                    .iter()
                    .position(|osc_app| osc_app.name == app.meta.name)
                {
                    app.osc_idx = osc_idx as i32;
                }
            }

            let displayed_apps = homebrew_apps
                .iter()
                .enumerate()
                .map(|(i, app)| DisplayedHomebrewApp::new(app, i))
                .collect::<Vec<_>>();

            displayed_homebrew_apps_clone.set_vec(displayed_apps);
        });

        let games_clone = games.clone();
        let weak = self.as_weak();
        self.on_load_game_info(move |i| {
            if let Some(disc_path) = games_clone.borrow()[i as usize].get_disc_path()
                && let Some(info) = DiscInfo::from_path(disc_path)
            {
                let info = DisplayedDiscInfo::from(&info);
                weak.upgrade().unwrap().set_current_disc_info(info);
            }
        });

        let games_clone = games.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_delete_game(move |i| {
            let res = {
                let game = &games_clone.borrow()[i as usize];
                fs::remove_dir_all(&game.path)
            };

            if let Err(e) = res {
                let msg = format!("Failed to delete game: {e}");
                notifications_clone.push(Notification::error(msg));
                return;
            }

            weak.upgrade().unwrap().invoke_refresh_all();
        });

        let homebrew_apps_clone = homebrew_apps.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_delete_homebrew_app(move |i| {
            let res = {
                let app = &homebrew_apps_clone.borrow()[i as usize];
                fs::remove_dir_all(&app.path)
            };

            if let Err(e) = res {
                let msg = format!("Failed to delete homebrew app: {e}");
                notifications_clone.push(Notification::error(msg));
                return;
            }

            weak.upgrade().unwrap().invoke_refresh_all();
        });

        let games_clone = games.clone();
        let weak = self.as_weak();
        let notifications_clone = notifications.clone();
        self.on_scrub_all_games(move || {
            let to_scrub = games_clone
                .borrow()
                .iter()
                .filter(|g| g.is_wii)
                .enumerate()
                .filter_map(|(i, game)| {
                    let disc_path = game.get_disc_path()?;
                    let mut f = File::open(disc_path).ok()?;
                    let meta = wii_disc_info::Meta::read(&mut f).ok()?;
                    let worth = meta.format() == wii_disc_info::Format::Wbfs
                        && is_worth_scrubbing(&mut f).ok()?;

                    worth.then_some(i as i32)
                })
                .collect::<Vec<_>>();

            if to_scrub.is_empty() {
                let msg = SharedString::from("No games need scrubbing");
                notifications_clone.push(Notification::info(msg));
                return;
            }

            let logic = weak.upgrade().unwrap();
            for i in to_scrub {
                logic.invoke_scrub_game(i);
            }
        });

        let notifications_clone = notifications.clone();
        let config_clone = config.clone();
        self.on_normalize_dir_layout(move || {
            let res = {
                let config = config_clone.borrow();
                normalize_dir_layout::perform(&config.contents.mount_point)
            };

            let msg = match &res {
                Ok(_) => SharedString::from("Directory layout successfully normalized"),
                Err(e) => slint::format!("Failed to normalize directory layout: {}", e),
            };

            let notification = match res {
                Ok(_) => Notification::info(msg),
                Err(_) => Notification::error(msg),
            };

            notifications_clone.push(notification);
        });

        let conversion_queue_clone = conversion_queue.clone();
        self.on_cancel_conversion(move |i| {
            let _ = conversion_queue_clone.remove(i as usize);
        });

        let conversion_queue_clone = conversion_queue.clone();
        self.on_cancel_all_conversions(move || {
            conversion_queue_clone.clear();
        });

        let weak = self.as_weak();
        self.on_check_for_updates(move || {
            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = twbm_core::updates::check();

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(Some(version)) => {
                        let version = slint::format!("v{version}");
                        logic.set_latest_version(version);
                    }
                    Ok(None) => {
                        eprintln!("No updates available");
                    }
                    Err(e) => {
                        let msg = slint::format!("Failed to check for updates: {e}");
                        logic.invoke_notify_error(msg);
                    }
                });
            });
        });

        let games_clone = games.clone();
        let config_clone = config.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_download_txtcodes(move |i| {
            let game_id = games_clone.borrow()[i as usize].id;
            let config = config_clone.borrow().clone();

            let msg = slint::format!("Downloading txtcodes for {game_id}");
            notifications_clone.push(Notification::info(msg));

            let weak = weak.clone();
            std::thread::spawn(move || {
                let res = twbm_core::txtcodes::download_cheats(game_id, &config);

                let _ = weak.upgrade_in_event_loop(move |logic| match res {
                    Ok(_) => {
                        let msg = slint::format!("Downloaded txtcodes for {game_id}");
                        logic.invoke_notify_info(msg);
                    }
                    Err(e) => {
                        let msg = slint::format!("Failed to download txtcodes for {game_id}: {e}");
                        logic.invoke_notify_error(msg);
                    }
                });
            });
        });

        #[cfg(target_os = "macos")]
        {
            let notifications_clone = notifications.clone();
            let config_clone = config.clone();

            self.on_run_dot_clean(move || {
                let res = {
                    let config = config_clone.borrow();
                    twbm_core::util::run_dot_clean(&config.contents.mount_point)
                };

                match res {
                    Ok(_) => {
                        let msg = "Successfully ran dot_clean".to_shared_string();
                        notifications_clone.push(Notification::info(msg));
                    }
                    Err(e) => {
                        let msg = slint::format!("Failed to run dot_clean: {e}");
                        notifications_clone.push(Notification::error(msg));
                    }
                }
            });
        }

        #[cfg(windows)]
        {
            self.on_set_window_color(move |is_dark| {
                crate::window_color::set(is_dark);
            });
        }

        // Initialize
        self.invoke_refresh_all();
    }
}
