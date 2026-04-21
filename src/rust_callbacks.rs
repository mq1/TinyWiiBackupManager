// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    ConversionKind, DriveInfo, Notification, QueuedArchiveConversion, QueuedConversion,
    QueuedScrubConversion, Rust, checksum, dialogs, game, homebrew_app, model::AppModel, osc,
    standard_conversion,
};
use slint::{Global, SharedString, ToSharedString, Window};
use std::path::Path;

impl Rust<'_> {
    pub fn register_callbacks(&self, state: &AppModel, window: &Window) {
        let state_clone = state.clone();
        self.on_open_that(move |uri| {
            if let Err(e) = open::that(uri) {
                state_clone.add_notification(e.into());
            }
        });

        let state_clone = state.clone();
        let window_handle = window.window_handle();
        self.on_pick_mount_point(move || {
            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                state_clone.set_mount_point(path);
            }
        });

        let state_clone = state.clone();
        self.on_set_wii_output_format(move |format| {
            state_clone.set_wii_output_format(format);
        });

        let state_clone = state.clone();
        self.on_set_gc_output_format(move |format| {
            state_clone.set_gc_output_format(format);
        });

        let state_clone = state.clone();
        self.on_set_always_split(move |always_split| {
            state_clone.set_always_split(always_split);
        });

        let state_clone = state.clone();
        self.on_set_scrub_update_partition(move |scrub_update_partition| {
            state_clone.set_scrub_update_partition(scrub_update_partition);
        });

        let state_clone = state.clone();
        self.on_set_remove_sources_games(move |remove_sources_games| {
            state_clone.set_remove_sources_games(remove_sources_games);
        });

        let state_clone = state.clone();
        self.on_set_remove_sources_apps(move |remove_sources_apps| {
            state_clone.set_remove_sources_apps(remove_sources_apps);
        });

        let state_clone = state.clone();
        self.on_set_txt_codes_source(move |source| {
            state_clone.set_txt_codes_source(source);
        });

        let state_clone = state.clone();
        self.on_set_theme_preference(move |theme_preference| {
            state_clone.set_theme_preference(theme_preference);
        });

        let state_clone = state.clone();
        self.on_set_view_as(move |format| {
            state_clone.set_view_as(format);
        });

        let state_clone = state.clone();
        self.on_set_sort_by(move |sort_by| {
            state_clone.set_sort_by(sort_by);
        });

        let state_clone = state.clone();
        self.on_set_show_wii(move |show_wii| {
            state_clone.set_show_wii(show_wii);
        });

        let state_clone = state.clone();
        self.on_set_show_gc(move |show_gc| {
            state_clone.set_show_gc(show_gc);
        });

        let state_clone = state.clone();
        self.on_refresh_all(move || {
            let (new_games, new_apps, drive_info) = {
                let config = state_clone.config().borrow();
                let root_path = Path::new(&config.contents.mount_point);

                let p = root_path.to_path_buf();
                let join = std::thread::spawn(move || DriveInfo::from_path(&p));

                let new_games = game::scan_drive(root_path);
                let new_apps = homebrew_app::scan_drive(root_path);

                let drive_info = join.join().unwrap();

                (new_games, new_apps, drive_info)
            };

            state_clone.set_games(new_games);
            state_clone.set_homebrew_apps(new_apps);
            state_clone.set_drive_info(drive_info);
        });

        let state_clone = state.clone();
        self.on_load_osc_apps(move |force_refresh| {
            let (new, h, min) = osc::load_contents(force_refresh).unwrap_or_default();
            state_clone.set_osc_apps(new);
            (h, min)
        });

        let state_clone = state.clone();
        self.on_filter_games(move |filter| {
            state_clone.set_games_filter(filter);
        });

        let state_clone = state.clone();
        self.on_filter_homebrew_apps(move |filter| {
            state_clone.set_homebrew_apps_filter(filter);
        });

        let state_clone = state.clone();
        self.on_filter_osc_apps(move |filter| {
            state_clone.set_osc_apps_filter(filter);
        });

        let state_clone = state.clone();
        self.on_close_notification(move |i| {
            #[allow(clippy::cast_sign_loss)]
            state_clone.close_notification(i as usize);
        });

        let weak = self.as_weak();
        self.on_checksum(move |game| {
            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                if let Err(e) = checksum::perform(&game.path, game.is_wii, &game.id, &weak) {
                    let _ = weak.upgrade_in_event_loop(move |rust| {
                        rust.invoke_notify_error(e.to_shared_string());
                    });
                }
            });
        });

        let state_clone = state.clone();
        self.on_notify_error(move |e| {
            state_clone.add_notification(Notification::error(e));
        });

        let state_clone = state.clone();
        let window_handle = window.window_handle();
        self.on_pick_games(move |recursively| {
            let paths = if recursively {
                dialogs::pick_games_r(&window_handle)
            } else {
                dialogs::pick_games(&window_handle)
            };

            let existing_ids = state_clone.existing_ids();

            let new = standard_conversion::make_queue(paths, &existing_ids);

            state_clone.set_conversion_queue_buffer(new);
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_confirm_conversion_queue_buffer(move || {
            state_clone.confirm_conversion_queue_buffer();

            if !state_clone.is_converting() {
                state_clone.set_is_converting(true);
                weak.upgrade().unwrap().invoke_trigger_conversion();
            }
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_trigger_conversion(move || {
            if let Some(mut conv) = state_clone.pop_conversion() {
                let weak = weak.clone();

                let _ = std::thread::spawn(move || {
                    let res = conv.perform(&weak);

                    let _ = weak.upgrade_in_event_loop(move |rust| {
                        rust.invoke_set_status(SharedString::new());
                        rust.invoke_set_crc32_status(SharedString::new());

                        if let Err(e) = res {
                            rust.invoke_notify_error(e.to_shared_string());
                        }

                        rust.invoke_trigger_conversion();
                    });
                });
            } else {
                state_clone.set_is_converting(false);
            }
        });

        let state_clone = state.clone();
        self.on_clear_conversion_queue_buffer(move || {
            state_clone.clear_conversion_queue_buffer();
        });

        let state_clone = state.clone();
        self.on_set_status(move |status| {
            state_clone.set_status(status);
        });

        let state_clone = state.clone();
        self.on_set_crc32_status(move |status| {
            state_clone.set_crc32_status(status);
        });

        let state_clone = state.clone();
        let window_handle = window.window_handle();
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

                state_clone.push_conversion(conv);
                if !state_clone.is_converting() {
                    state_clone.set_is_converting(true);
                    weak.upgrade().unwrap().invoke_trigger_conversion();
                }
            }
        });

        let state_clone = state.clone();
        let weak = self.as_weak();
        self.on_scrub_game(move |game| {
            let scrub = QueuedScrubConversion { game };

            let conv = QueuedConversion {
                kind: ConversionKind::Scrub,
                scrub,
                ..Default::default()
            };

            state_clone.push_conversion(conv);
            if !state_clone.is_converting() {
                state_clone.set_is_converting(true);
                weak.upgrade().unwrap().invoke_trigger_conversion();
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
