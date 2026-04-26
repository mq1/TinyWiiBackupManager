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
use std::{cell::RefCell, path::Path, rc::Rc};
use twbm_core::{
    app_state::AppState, checksum, config::Config, disc_info::DiscInfo, drive_info::DriveInfo,
    game_id::GameID,
};

impl Logic<'_> {
    pub fn init(&self, config: Config, window: &Window) {
        let displayed_config = DisplayedConfig::from(&config);
        let state = AppState::new(config);

        let games = Rc::new(VecModel::from(Vec::new()));
        let games_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_games = Rc::new(SortModel::new(
            games.clone(),
            games::get_compare_fn(state.config().clone()),
        ));
        let filtered_games = Rc::new(FilterModel::new(
            sorted_games.clone(),
            games::get_filter_fn(games_filter.clone(), state.config().clone()),
        ));

        let homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let sorted_homebrew_apps = Rc::new(SortModel::new(
            homebrew_apps.clone(),
            homebrew_apps::get_compare_fn(state.config().clone()),
        ));
        let filtered_homebrew_apps = Rc::new(FilterModel::new(
            sorted_homebrew_apps.clone(),
            homebrew_apps::get_filter_fn(homebrew_apps_filter.clone()),
        ));

        let osc_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let filtered_osc_apps = Rc::new(FilterModel::new(
            osc_apps.clone(),
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

        let state_clone = state.clone();
        let weak = self.as_weak();
        let notifications_clone = notifications.clone();
        self.on_sync_config(move || {
            let logic = weak.upgrade().unwrap();
            let config = state_clone.config().borrow();
            let displayed_config = DisplayedConfig::from(&*config);

            logic.set_config(displayed_config);
            if let Err(e) = config.write() {
                notifications_clone.push(e.into());
            }
        });

        let notifications_clone = notifications.clone();
        self.on_open_that(move |uri| {
            if let Err(e) = open::that(uri) {
                notifications_clone.push(e.into());
            }
        });

        let weak = self.as_weak();
        let window_handle = window.window_handle();
        let state_clone = state.clone();
        self.on_pick_mount_point(move || {
            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                let mut config = state_clone.config().borrow_mut();
                config.contents.mount_point = path;

                let logic = weak.upgrade().unwrap();
                logic.invoke_sync_config();
                logic.invoke_refresh_all();
            }
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_wii_output_format(move |format| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.wii_output_format = format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_gc_output_format(move |format| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.gc_output_format = format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_always_split(move |always_split| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.always_split = always_split;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_scrub_update_partition(move |scrub_update_partition| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.scrub_update_partition = scrub_update_partition;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_remove_sources_games(move |remove_sources_games| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.remove_sources_games = remove_sources_games;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_remove_sources_apps(move |remove_sources_apps| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.remove_sources_apps = remove_sources_apps;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_txt_codes_source(move |source| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.txt_codes_source = source.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_theme_preference(move |theme_preference| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.theme_preference = theme_preference.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_set_view_as(move |format| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.view_as = format.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let state_clone = state.clone();
        let sorted_games_clone = sorted_games.clone();
        let sorted_homebrew_apps_clone = sorted_homebrew_apps.clone();
        let weak = self.as_weak();
        self.on_set_sort_by(move |sort_by| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.sort_by = sort_by.try_into().unwrap_or_default();
            weak.upgrade().unwrap().invoke_sync_config();

            sorted_games_clone.reset();
            sorted_homebrew_apps_clone.reset();
        });

        let state_clone = state.clone();
        let filtered_games_clone = filtered_games.clone();
        let weak = self.as_weak();
        self.on_set_show_wii(move |show_wii| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.show_wii = show_wii;
            weak.upgrade().unwrap().invoke_sync_config();

            filtered_games_clone.reset();
        });

        let state_clone = state.clone();
        let filtered_games_clone = filtered_games.clone();
        let weak = self.as_weak();
        self.on_set_show_gc(move |show_gc| {
            let mut config = state_clone.config().borrow_mut();
            config.contents.show_gc = show_gc;
            weak.upgrade().unwrap().invoke_sync_config();

            filtered_games_clone.reset();
        });

        let games_clone = games.clone();
        let homebrew_apps_clone = homebrew_apps.clone();
        let state_clone = state.clone();
        let weak = self.as_weak();
        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_refresh_all(move || {
            let logic = weak.upgrade().unwrap();

            let (new_games, new_apps, drive_info) = {
                let config = state_clone.config().borrow_mut();
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

            state_clone.set_games(new_games);
            state_clone.set_homebrew_apps(new_apps);
            state_clone.set_drive_info(drive_info);

            games_clone.set_vec(displayed_games);
            homebrew_apps_clone.set_vec(displayed_apps);
            logic.set_drive_info(displayed_drive_info);

            let mut is_downloading_covers = is_downloading_covers_clone.borrow_mut();
            if !*is_downloading_covers {
                *is_downloading_covers = true;

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    if let Err(e) = covers::download_covers(ids, weak.clone()) {
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

        let state_clone = state.clone();
        let osc_apps_clone = osc_apps.clone();
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

            state_clone.set_osc_apps(new);

            osc_apps_clone.set_vec(displayed_apps);
            logic.set_osc_refreshed_x_hours_ago(hours);
            logic.set_osc_refreshed_x_minutes_ago(minutes);

            logic.invoke_pair_homebrew_osc();
        });

        let is_downloading_osc_icons_clone = is_downloading_osc_icons.clone();
        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_download_osc_icons(move || {
            let mut is_downloading_osc_icons = is_downloading_osc_icons_clone.borrow_mut();
            if !*is_downloading_osc_icons {
                *is_downloading_osc_icons = true;

                let apps = state_clone.osc_apps().borrow().clone();

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    osc::download_icons(&apps, weak);
                });
            }
        });

        let osc_apps_clone = osc_apps.clone();
        self.on_reload_osc_icon(move |i| {
            let mut app = osc_apps_clone.row_data(i as usize).unwrap();
            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.slug));

            if let Ok(icon) = Image::load_from_path(&icon_path) {
                app.icon = icon;
                osc_apps_clone.set_row_data(i as usize, app);
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

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_checksum(move |i| {
            let game = state_clone.games().borrow()[i as usize].clone();
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
        let state_clone = state.clone();
        let conversion_queue_buffer_clone = conversion_queue_buffer.clone();
        self.on_pick_games(move |recursively| {
            let paths = if recursively {
                dialogs::pick_games_r(&window_handle)
            } else {
                dialogs::pick_games(&window_handle)
            };

            let existing_ids = state_clone
                .games()
                .borrow()
                .iter()
                .map(|g| g.id)
                .collect::<Vec<_>>();

            let mut new = Vec::new();
            for path in paths {
                if let Some(info) = DiscInfo::from_path(path)
                    && let Some(game_id) = GameID::new(info.meta.game_id())
                    && existing_ids.iter().all(|id| *id != game_id)
                {
                    new.push(QueuedConversion {
                        kind: ConversionKind::Standard,
                        path: info.path.to_string_lossy().to_shared_string(),
                        game_idx: -1,
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

        let state_clone = state.clone();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_trigger_conversion(move || {
            if conversion_queue_clone.row_count() == 0 {
                *is_converting_clone.borrow_mut() = false;
                notifications_clone.push(Notification::info("Conversion queue empty"));
                return;
            }

            let queued = conversion_queue_clone.remove(0);
            let conv = Conversion::new(&queued, &state_clone.games().borrow());

            let weak = weak.clone();
            let drive_info = *state_clone.drive_info().borrow();
            let config = state_clone.config().borrow().clone();

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

        let state_clone = state.clone();
        let window_handle = window.window_handle();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let weak = self.as_weak();
        self.on_archive_game(move |i| {
            let game = &state_clone.games().borrow()[i as usize];
            let out_path = dialogs::save_game(&window_handle, game);

            if let Some(out_path) = out_path {
                let queued = QueuedConversion {
                    kind: ConversionKind::Archive,
                    game_idx: i,
                    path: out_path.to_string_lossy().to_shared_string(),
                };

                conversion_queue_clone.push(queued);

                let mut is_converting = is_converting_clone.borrow_mut();
                if !*is_converting {
                    *is_converting = true;
                    weak.upgrade().unwrap().invoke_trigger_conversion();
                }
            }
        });

        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let weak = self.as_weak();
        self.on_scrub_game(move |i| {
            let conv = QueuedConversion {
                kind: ConversionKind::Scrub,
                game_idx: i,
                path: SharedString::new(),
            };

            conversion_queue_clone.push(conv);

            let mut is_converting = is_converting_clone.borrow_mut();
            if !*is_converting {
                *is_converting = true;
                weak.upgrade().unwrap().invoke_trigger_conversion();
            }
        });

        let state_clone = state.clone();
        let window_handle = window.window_handle();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_pick_homebrew_apps(move || {
            let paths = dialogs::pick_homebrew_apps(&window_handle);
            let root_dir = &state_clone.config().borrow().contents.mount_point;

            if let Err(e) = twbm_core::util::install_zips(root_dir, &paths) {
                notifications_clone.push(e.into());
            } else {
                let msg = format!("{} apps installed successfully", paths.len());
                notifications_clone.push(Notification::info(msg));
                weak.upgrade().unwrap().invoke_refresh_all();
            }
        });

        let state_clone = state.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_install_osc_app(move |i| {
            let app = state_clone.osc_apps().borrow()[i as usize].clone();
            let root_dir = state_clone.config().borrow().contents.mount_point.clone();

            notifications_clone.push(Notification::info(format!("Installing {}", &app.name)));

            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = app.install(&root_dir);

                if let Err(e) = res {
                    let _ = weak.upgrade_in_event_loop(move |logic| {
                        logic.invoke_notify_error(e.to_shared_string());
                    });
                } else {
                    let msg = format!("{} installed successfully", &app.name);
                    let _ = weak.upgrade_in_event_loop(move |logic| {
                        logic.invoke_notify_info(msg.to_shared_string());
                        logic.invoke_refresh_all();
                    });
                }
            });
        });

        let games_clone = games.clone();
        self.on_reload_cover(move |i| {
            let mut game = games_clone.row_data(i as usize).unwrap();
            let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

            if let Ok(cover) = Image::load_from_path(&cover_path) {
                game.cover = cover;
                games_clone.set_row_data(i as usize, game);
            }
        });

        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_finished_downloading_covers(move || {
            *is_downloading_covers_clone.borrow_mut() = false;
        });

        let state_clone = state.clone();
        let homebrew_apps_clone = homebrew_apps.clone();
        self.on_pair_homebrew_osc(move || {
            let mut apps = state_clone.homebrew_apps().borrow_mut();

            for app in apps.iter_mut() {
                if let Some(osc_idx) = osc_apps
                    .iter()
                    .position(|osc_app| osc_app.name == app.meta.name)
                {
                    app.osc_idx = osc_idx as i32;
                }
            }

            let displayed_apps = apps
                .iter()
                .enumerate()
                .map(|(i, app)| DisplayedHomebrewApp::new(app, i))
                .collect::<Vec<_>>();

            homebrew_apps_clone.set_vec(displayed_apps);
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_load_game_info(move |i| {
            let game = &state_clone.games().borrow()[i as usize];

            if let Some(disc_path) = game.get_disc_path()
                && let Some(info) = DiscInfo::from_path(disc_path)
            {
                let info = DisplayedDiscInfo::from(&info);
                weak.upgrade().unwrap().set_current_disc_info(info);
            }
        });

        #[cfg(windows)]
        {
            let window_handle = window.window_handle();
            self.on_set_window_color(move |is_dark| {
                crate::window_color::set(&window_handle, is_dark);
            });
        }
    }
}
