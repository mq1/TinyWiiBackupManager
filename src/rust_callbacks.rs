// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Rust, dialogs, game, homebrew_app, model::AppModel, osc};
use slint::Window;
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
        self.on_set_view_as(move |view_as| {
            state_clone.set_view_as(view_as);
        });

        let state_clone = state.clone();
        self.on_set_sort_by(move |sort_by| {
            state_clone.set_sort_by(sort_by);
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
        self.on_set_sort_by(move |sort_by| state_clone.set_sort_by(sort_by));

        let state_clone = state.clone();
        self.on_set_show_wii(move |show_wii| state_clone.set_show_wii(show_wii));

        let state_clone = state.clone();
        self.on_set_show_gc(move |show_gc| state_clone.set_show_gc(show_gc));

        let state_clone = state.clone();
        self.on_close_notification(move |i| {
            state_clone.close_notification(i as usize);
        });

        #[cfg(windows)]
        {
            let weak = app.as_weak();
            self.on_set_window_color(move |is_dark| {
                let app = weak.upgrade().unwrap();
                window_color::set(app.window(), is_dark);
            });
        }
    }
}
