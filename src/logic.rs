// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    Config, ConversionKind, DiscInfo, DriveInfo, Logic, Notification, QueuedArchiveConversion, QueuedConversion, QueuedScrubConversion, checksum, convert::Conversion, covers, data_dir::DATA_DIR, dialogs, game, homebrew_app, osc, standard_conversion, util
};
use arrayvec::ArrayString;
use slint::{
    FilterModel, Global, Image, Model, ModelRc, SharedString, SortModel, ToSharedString, VecModel,
    Window,
};
use std::{
    cell::RefCell, fs, path::{Path, PathBuf}, rc::Rc
};

impl Logic<'_> {
    pub fn init(&self, config: Config, window: &Window) {
        // MODEL

        self.set_config(config.clone());
        let config = Rc::new(RefCell::new(config));

        let games = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps = Rc::new(VecModel::from(Vec::new()));

        let sorted_games = Rc::new(SortModel::new(
            games.clone(),
            game::get_compare_fn(config.clone()),
        ));
        let sorted_homebrew_apps = Rc::new(SortModel::new(
            homebrew_apps.clone(),
            homebrew_app::get_compare_fn(config.clone()),
        ));

        let games_filter = Rc::new(RefCell::new(SharedString::new()));
        let homebrew_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let osc_apps_filter = Rc::new(RefCell::new(SharedString::new()));

        let filtered_games = Rc::new(FilterModel::new(
            sorted_games.clone(),
            game::get_filter_fn(games_filter.clone(), config.clone()),
        ));
        let filtered_homebrew_apps = Rc::new(FilterModel::new(
            sorted_homebrew_apps.clone(),
            homebrew_app::get_filter_fn(homebrew_apps_filter.clone()),
        ));
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
        self.set_games(ModelRc::from(filtered_games.clone()));
        self.set_homebrew_apps(ModelRc::from(filtered_homebrew_apps.clone()));
        self.set_osc_apps(ModelRc::from(filtered_osc_apps.clone()));
        self.set_notifications(ModelRc::from(notifications.clone()));
        self.set_conversion_queue(ModelRc::from(conversion_queue.clone()));
        self.set_conversion_queue_buffer(ModelRc::from(conversion_queue_buffer.clone()));

        // UPDATE

        let config_clone = config.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_sync_config(move || {
            let logic = weak.upgrade().unwrap();
            let config = config_clone.borrow();

            logic.set_config(config.clone());
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
        let config_clone = config.clone();
        self.on_pick_mount_point(move || {
            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                config_clone.borrow_mut().contents.mount_point =
                    path.to_string_lossy().to_shared_string();

                let logic = weak.upgrade().unwrap();
                logic.invoke_sync_config();
                logic.invoke_refresh_all();
            }
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_wii_output_format(move |format| {
            config_clone.borrow_mut().contents.wii_output_format = format;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_gc_output_format(move |format| {
            config_clone.borrow_mut().contents.gc_output_format = format;
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
            config_clone.borrow_mut().contents.txt_codes_source = source;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_theme_preference(move |theme_preference| {
            config_clone.borrow_mut().contents.theme_preference = theme_preference;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_set_view_as(move |format| {
            config_clone.borrow_mut().contents.view_as = format;
            weak.upgrade().unwrap().invoke_sync_config();
        });

        let config_clone = config.clone();
        let sorted_games_clone = sorted_games.clone();
        let sorted_homebrew_apps_clone = sorted_homebrew_apps.clone();
        let weak = self.as_weak();
        self.on_set_sort_by(move |sort_by| {
            config_clone.borrow_mut().contents.sort_by = sort_by;
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

        let games_clone = games.clone();
        let homebrew_apps_clone = homebrew_apps.clone();
        let config_clone = config.clone();
        let weak = self.as_weak();
        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_refresh_all(move || {
            let logic = weak.upgrade().unwrap();

            let (new_games, new_apps, drive_info) = {
                let config = config_clone.borrow();
                let root_path = Path::new(&config.contents.mount_point);

                let p = root_path.to_path_buf();
                let join = std::thread::spawn(move || DriveInfo::from_path(&p));

                let new_games = game::scan_drive(root_path);
                let new_apps = homebrew_app::scan_drive(root_path);

                let drive_info = join.join().unwrap();

                (new_games, new_apps, drive_info)
            };

            let ids = new_games
                .iter()
                .filter_map(|g| ArrayString::from(&g.id).ok())
                .collect::<Vec<_>>();

            games_clone.set_vec(new_games);
            homebrew_apps_clone.set_vec(new_apps);
            logic.set_drive_info(drive_info);

            let mut is_downloading_covers = is_downloading_covers_clone.borrow_mut();
            if !*is_downloading_covers {
                *is_downloading_covers = true;
                covers::download_covers(ids, weak.clone());
            }

            logic.invoke_pair_homebrew_osc();
        });

        let weak = self.as_weak();
        self.on_cache_osc_contents(move |force_refresh| {
            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = osc::cache_contents(force_refresh);

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
        let weak = self.as_weak();
        self.on_osc_contents_cached(move || {
            let logic = weak.upgrade().unwrap();

            let (new, hours, minutes) = osc::load_contents().unwrap_or_default();

            osc_apps_clone.set_vec(new);
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

                let icon_urls = osc_apps_clone
                    .iter()
                    .map(|app| {
                        (
                            app.meta.slug.to_string(),
                            app.meta.assets.icon.url.to_string(),
                        )
                    })
                    .collect::<Vec<_>>();

                let weak = weak.clone();
                let _ = std::thread::spawn(move || {
                    osc::download_icons(icon_urls, weak);
                });
            }
        });

        let osc_apps_clone = osc_apps.clone();
        self.on_reload_osc_icon(move |i| {
            #[allow(clippy::cast_sign_loss)]
            let i = i as usize;

            let mut app = osc_apps_clone.row_data(i).unwrap();
            let icon_path = DATA_DIR.join(format!("osc-icons/{}.png", &app.meta.slug));

            if let Ok(icon) = Image::load_from_path(&icon_path) {
                app.icon = icon;
                osc_apps_clone.set_row_data(i, app);
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
            #[allow(clippy::cast_sign_loss)]
            notifications_clone.remove(i as usize);
        });

        let weak = self.as_weak();
        self.on_checksum(move |game| {
            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                if let Err(e) = checksum::perform(&game.path, game.is_wii, &game.id, &weak) {
                    let _ = weak.upgrade_in_event_loop(move |logic| {
                        logic.invoke_notify_error(e.to_shared_string());
                    });
                }
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
                .iter()
                .map(|g| g.id.to_string())
                .collect::<Vec<_>>();
            let new = standard_conversion::make_queue(paths, &existing_ids);
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

        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let config_clone = config.clone();
        let weak = self.as_weak();
        self.on_trigger_conversion(move || {
            if conversion_queue_clone.row_count() == 0 {
                *is_converting_clone.borrow_mut() = false;
                return;
            }

            let conv = conversion_queue_clone.remove(0);
            let config = config_clone.borrow();
            let drive_info = weak.upgrade().unwrap().get_drive_info();
            let mut conv = Conversion::new(&conv, &config.contents, &drive_info);

            let weak = weak.clone();

            let _ = std::thread::spawn(move || {
                let res = conv.perform(&weak);

                let _ = weak.upgrade_in_event_loop(move |logic| {
                    logic.invoke_set_status(SharedString::new());

                    if let Err(e) = res {
                        logic.invoke_notify_error(e.to_shared_string());
                    }

                    logic.invoke_refresh_all();
                    logic.invoke_trigger_conversion();
                });
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

        let window_handle = window.window_handle();
        let conversion_queue_clone = conversion_queue.clone();
        let is_converting_clone = is_converting.clone();
        let weak = self.as_weak();
        self.on_archive_game(move |game| {
            let out_path = dialogs::save_game(&window_handle, &game);

            if let Some(out_path) = out_path {
                let archive = QueuedArchiveConversion {
                    game_title: game.title.clone(),
                    in_path: game.path.clone(),
                    out_path: out_path.to_string_lossy().to_shared_string(),
                };

                let conv = QueuedConversion {
                    kind: ConversionKind::Archive,
                    archive,
                    ..Default::default()
                };

                conversion_queue_clone.push(conv);

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
        self.on_scrub_game(move |game| {
            let scrub = QueuedScrubConversion { game };

            let conv = QueuedConversion {
                kind: ConversionKind::Scrub,
                scrub,
                ..Default::default()
            };

            conversion_queue_clone.push(conv);

            let mut is_converting = is_converting_clone.borrow_mut();
            if !*is_converting {
                *is_converting = true;
                weak.upgrade().unwrap().invoke_trigger_conversion();
            }
        });

        let window_handle = window.window_handle();
        let config_clone = config.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_pick_homebrew_apps(move || {
            let paths = dialogs::pick_homebrew_apps(&window_handle);
            let config = config_clone.borrow();
            let root_dir = Path::new(&config.contents.mount_point);

            if let Err(e) = util::install_zips(root_dir, &paths) {
                notifications_clone.push(e.into());
            } else {
                let msg = format!("{} apps installed successfully", paths.len());
                notifications_clone.push(Notification::info(msg));
                weak.upgrade().unwrap().invoke_refresh_all();
            }
        });

        let config_clone = config.clone();
        let notifications_clone = notifications.clone();
        let weak = self.as_weak();
        self.on_install_osc_app(move |app| {
            let config = config_clone.borrow();
            let root_dir = PathBuf::from(&config.contents.mount_point);

            notifications_clone.push(Notification::info(format!("Installing {}", &app.meta.name)));

            let weak = weak.clone();

            std::thread::spawn(move || {
                let res = util::install_zip_from_url(&app.meta.assets.archive.url, &root_dir);

                if let Err(e) = res {
                    let _ = weak.upgrade_in_event_loop(move |logic| {
                        logic.invoke_notify_error(e.to_shared_string());
                    });
                } else {
                    let msg = format!("{} installed successfully", &app.meta.name);
                    let _ = weak.upgrade_in_event_loop(move |logic| {
                        logic.invoke_notify_info(msg.to_shared_string());
                        logic.invoke_refresh_all();
                    });
                }
            });
        });

        let games_clone = games.clone();
        self.on_reload_cover(move |i| {
            #[allow(clippy::cast_sign_loss)]
            let i = i as usize;

            let mut game = games_clone.row_data(i).unwrap();
            let cover_path = DATA_DIR.join(format!("covers/{}.png", &game.id));

            if let Ok(cover) = Image::load_from_path(&cover_path) {
                game.cover = cover;
                games_clone.set_row_data(i, game);
            }
        });

        let is_downloading_covers_clone = is_downloading_covers.clone();
        self.on_finished_downloading_covers(move || {
            *is_downloading_covers_clone.borrow_mut() = false;
        });

        let homebrew_apps_clone = homebrew_apps.clone();
        let osc_apps_clone = osc_apps.clone();
        self.on_pair_homebrew_osc(move || {
            let mut homebrew_apps = homebrew_apps_clone.iter().collect::<Vec<_>>();
            let osc_apps = osc_apps_clone.iter().collect::<Vec<_>>();

            for app in &mut homebrew_apps {
                if let Some(osc_i) = osc_apps.iter().position(|a| a.meta.slug == app.slug) {
                    app.osc_i = osc_i as i32;
                }
            }

            homebrew_apps_clone.set_vec(homebrew_apps);
        });

        let games_clone = games.clone();
        self.on_load_game_info(move |i| {
            #[allow(clippy::cast_sign_loss)]
            let i = i as usize;

            let mut game = games_clone.row_data(i).unwrap();

            let game_dir = Path::new(&game.path);
            match DiscInfo::try_from_game_dir(game_dir) {
                Ok(disc_info) => {
                    game.disc_info = disc_info;
                },
                Err(e) => {
                    game.disc_info_err = e.to_shared_string();
                }
            };

            let crc32_path = game_dir.join(format!("{}.crc32", &game.id));
            game.crc32 = fs::read_to_string(&crc32_path).unwrap_or_default().to_shared_string();

            games_clone.set_row_data(i, game);
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
