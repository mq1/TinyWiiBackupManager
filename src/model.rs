// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, Config, Game, GcOutputFormat, HomebrewApp, Notification, OscApp, QueuedConversion,
    SortBy, ThemePreference, TxtCodesSource, ViewAs, WiiOutputFormat, game, homebrew_app,
    mirrored::Mirrored, osc,
};
use slint::{FilterModel, ModelRc, SharedString, SortModel, ToSharedString, VecModel};
use std::{
    cell::{Ref, RefCell},
    cmp::Ordering,
    path::PathBuf,
    rc::Rc,
};

type SortedModel<T> = SortModel<Rc<VecModel<T>>, Box<dyn Fn(&T, &T) -> Ordering>>;
type FilteredModel<T> = FilterModel<Rc<SortedModel<T>>, Box<dyn Fn(&T) -> bool>>;
type JustFilteredModel<T> = FilterModel<Rc<VecModel<T>>, Box<dyn Fn(&T) -> bool>>;

#[derive(Clone)]
pub struct AppModel {
    config: Rc<Mirrored<Config, AppWindow>>,
    status: Rc<Mirrored<SharedString, AppWindow>>,
    crc32_status: Rc<Mirrored<SharedString, AppWindow>>,

    games: Rc<VecModel<Game>>,
    homebrew_apps: Rc<VecModel<HomebrewApp>>,
    osc_apps: Rc<VecModel<OscApp>>,

    sorted_games: Rc<SortedModel<Game>>,
    sorted_homebrew_apps: Rc<SortedModel<HomebrewApp>>,

    games_filter: Rc<RefCell<SharedString>>,
    homebrew_apps_filter: Rc<RefCell<SharedString>>,
    osc_apps_filter: Rc<RefCell<SharedString>>,

    filtered_games: Rc<FilteredModel<Game>>,
    filtered_homebrew_apps: Rc<FilteredModel<HomebrewApp>>,
    filtered_osc_apps: Rc<JustFilteredModel<OscApp>>,

    notifications: Rc<VecModel<Notification>>,

    conversion_queue: Rc<VecModel<QueuedConversion>>,
    conversion_queue_buffer: Rc<VecModel<QueuedConversion>>,
}

impl AppModel {
    pub fn new(config: Config, app: &AppWindow) -> Self {
        let config = Rc::new(Mirrored::new(config, app, AppWindow::set_config));
        let status = Rc::new(Mirrored::new(
            SharedString::new(),
            app,
            AppWindow::set_status,
        ));
        let crc32_status = Rc::new(Mirrored::new(
            SharedString::new(),
            app,
            AppWindow::set_crc32_status,
        ));

        let games = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps = Rc::new(VecModel::from(Vec::new()));

        let compare_games = game::get_compare_fn(config.clone());
        let compare_homebrew_apps = homebrew_app::get_compare_fn(config.clone());

        let sorted_games = Rc::new(SortModel::new(games.clone(), compare_games));
        let sorted_homebrew_apps =
            Rc::new(SortModel::new(homebrew_apps.clone(), compare_homebrew_apps));

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

        app.set_games(ModelRc::from(filtered_games.clone()));
        app.set_homebrew_apps(ModelRc::from(filtered_homebrew_apps.clone()));
        app.set_osc_apps(ModelRc::from(filtered_osc_apps.clone()));
        app.set_notifications(ModelRc::from(notifications.clone()));
        app.set_conversion_queue(ModelRc::from(conversion_queue.clone()));
        app.set_conversion_queue_buffer(ModelRc::from(conversion_queue_buffer.clone()));

        Self {
            config,
            status,
            crc32_status,
            games,
            homebrew_apps,
            osc_apps,
            sorted_games,
            sorted_homebrew_apps,
            games_filter,
            homebrew_apps_filter,
            osc_apps_filter,
            filtered_games,
            filtered_homebrew_apps,
            filtered_osc_apps,
            notifications,
            conversion_queue,
            conversion_queue_buffer,
        }
    }

    pub fn borrow_config(&self) -> Ref<'_, Config> {
        self.config.borrow()
    }

    pub fn set_mount_point(&self, mount_point: PathBuf) {
        self.config.edit(|config| {
            config.contents.mount_point = mount_point.to_string_lossy().to_shared_string();
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_wii_output_format(&self, wii_output_format: WiiOutputFormat) {
        self.config.edit(|config| {
            config.contents.wii_output_format = wii_output_format;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_gc_output_format(&self, gc_output_format: GcOutputFormat) {
        self.config.edit(|config| {
            config.contents.gc_output_format = gc_output_format;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_always_split(&self, always_split: bool) {
        self.config.edit(|config| {
            config.contents.always_split = always_split;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_scrub_update_partition(&self, scrub_update_partition: bool) {
        self.config.edit(|config| {
            config.contents.scrub_update_partition = scrub_update_partition;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_remove_sources_games(&self, remove_sources_games: bool) {
        self.config.edit(|config| {
            config.contents.remove_sources_games = remove_sources_games;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_remove_sources_apps(&self, remove_sources_apps: bool) {
        self.config.edit(|config| {
            config.contents.remove_sources_apps = remove_sources_apps;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_txt_codes_source(&self, txt_codes_source: TxtCodesSource) {
        self.config.edit(|config| {
            config.contents.txt_codes_source = txt_codes_source;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_theme_preference(&self, theme_preference: ThemePreference) {
        self.config.edit(|config| {
            config.contents.theme_preference = theme_preference;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_view_as(&self, view_as: ViewAs) {
        self.config.edit(|config| {
            config.contents.view_as = view_as;
        });

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_sort_by(&self, sort_by: SortBy) {
        self.config.edit(|config| {
            config.contents.sort_by = sort_by;
        });

        self.sorted_games.reset();
        self.sorted_homebrew_apps.reset();

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_show_wii(&self, show_wii: bool) {
        self.config.edit(|config| {
            config.contents.show_wii = show_wii;
        });

        self.filtered_games.reset();

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_show_gc(&self, show_gc: bool) {
        self.config.edit(|config| {
            config.contents.show_gc = show_gc;
        });

        self.filtered_games.reset();

        if let Err(e) = self.config.borrow().write() {
            self.add_notification(e.into());
        }
    }

    pub fn set_games_filter(&self, filter: SharedString) {
        *self.games_filter.borrow_mut() = filter;
        self.filtered_games.reset();
    }

    pub fn set_homebrew_apps_filter(&self, filter: SharedString) {
        *self.homebrew_apps_filter.borrow_mut() = filter;
        self.filtered_homebrew_apps.reset();
    }

    pub fn set_osc_apps_filter(&self, filter: SharedString) {
        *self.osc_apps_filter.borrow_mut() = filter;
        self.filtered_osc_apps.reset();
    }

    pub fn set_games(&self, games: Vec<Game>) {
        self.games.clear();
        self.games.extend(games);
    }

    pub fn set_homebrew_apps(&self, homebrew_apps: Vec<HomebrewApp>) {
        self.homebrew_apps.clear();
        self.homebrew_apps.extend(homebrew_apps);
    }

    pub fn set_osc_apps(&self, osc_apps: Vec<OscApp>) {
        self.osc_apps.clear();
        self.osc_apps.extend(osc_apps);
    }

    pub fn add_notification(&self, notification: Notification) {
        self.notifications.push(notification);
    }

    pub fn close_notification(&self, index: usize) {
        self.notifications.remove(index);
    }

    pub fn add_conversions_to_queue(
        &self,
        conversions: impl IntoIterator<Item = QueuedConversion>,
    ) {
        self.conversion_queue.extend(conversions);
    }

    pub fn remove_queued_conversion(&self, index: usize) {
        self.conversion_queue.remove(index);
    }

    pub fn clear_conversion_queue(&self) {
        self.conversion_queue.clear();
    }
}
