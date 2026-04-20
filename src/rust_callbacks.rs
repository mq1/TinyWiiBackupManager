// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Rust, dialogs, game, homebrew_app, model::AppModel, osc};
use std::path::Path;

impl Rust<'_> {
    pub fn register_callbacks(&self, state: &AppModel, window: &slint::Window) {
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
        self.on_refresh_all(move || {
            let (new_games, new_apps) = {
                let config = state_clone.borrow_config();
                let root_path = Path::new(&config.contents.mount_point);
                let new_games = game::scan_drive(root_path);
                let new_apps = homebrew_app::scan_drive(root_path);

                (new_games, new_apps)
            };

            state_clone.set_games(new_games);
            state_clone.set_homebrew_apps(new_apps);
        });

        let state_clone = state.clone();
        self.on_load_osc_apps(move |force_refresh| {
            let (new, h, min) = osc::load_contents(force_refresh).unwrap_or_default();
            state_clone.set_osc_apps(new);
            (h, min)
        });

        let state_clone = state.clone();
        self.on_filter_games(move |filter| state_clone.set_games_filter(filter));

        let state_clone = state.clone();
        self.on_filter_homebrew_apps(move |filter| state_clone.set_homebrew_apps_filter(filter));

        let state_clone = state.clone();
        self.on_filter_osc_apps(move |filter| state_clone.set_osc_apps_filter(filter));

        let state_clone = state.clone();
        self.on_close_notification(move |i| {
            #[allow(clippy::cast_sign_loss)]
            state_clone.close_notification(i as usize);
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
