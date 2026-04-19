// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{AppWindow, Rust, dialogs, game, homebrew_app, model::AppModel, osc};
use slint::ComponentHandle;
use std::path::Path;

#[cfg(windows)]
mod window_color;

#[cfg(windows)]
mod xp_dialogs;

impl Rust<'_> {
    pub fn register_callbacks(&self, state: &AppModel, app: &AppWindow) {
        let state_clone = state.clone();
        self.on_open_that(move |uri| {
            if let Err(e) = open::that(uri) {
                state_clone.add_notification(e.into());
            }
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_pick_mount_point(move || {
            let app = weak.upgrade().unwrap();
            let window_handle = app.window().window_handle();

            if let Some(path) = dialogs::pick_mount_point(&window_handle) {
                state_clone.set_mount_point(path);
            }

            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_set_view_as(move |view_as| {
            state_clone.set_view_as(view_as);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_set_sort_by(move |sort_by| {
            state_clone.set_sort_by(sort_by);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        self.on_load_games(move |path| {
            let path = Path::new(&path);
            let new = game::scan_drive(path);
            state_clone.set_games(new);
        });

        let state_clone = state.clone();
        self.on_load_homebrew_apps(move |path| {
            let path = Path::new(&path);
            let new = homebrew_app::scan_drive(path).unwrap_or_default();
            state_clone.set_homebrew_apps(new);
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
        let weak = app.as_weak();
        self.on_set_sort_by(move |sort_by| {
            state_clone.set_sort_by(sort_by);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_set_show_wii(move |show_wii| {
            state_clone.set_show_wii(show_wii);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_set_show_gc(move |show_gc| {
            state_clone.set_show_gc(show_gc);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());
        });

        let state_clone = state.clone();
        self.on_close_notification(move |i| {
            #[allow(clippy::cast_possible_truncation)]
            state_clone.close_notification(i as usize);
        });

        let state_clone = state.clone();
        let weak = app.as_weak();
        self.on_set_theme_preference(move |theme_preference| {
            state_clone.set_theme_preference(theme_preference);

            let app = weak.upgrade().unwrap();
            app.set_config(state_clone.config());

            #[cfg(windows)]
            {
                let window_handle = app.window().window_handle();
                window_color::set(&window_handle, theme_preference);
            }
        });
    }
}
