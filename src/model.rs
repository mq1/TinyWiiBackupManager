// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, Config, Game, HomebrewApp, Notification, OscApp, QueuedConversion, SortBy, ViewAs,
    game, homebrew_app, osc,
};
use slint::{FilterModel, ModelRc, SharedString, SortModel, ToSharedString, VecModel};
use std::{cell::RefCell, cmp::Ordering, path::PathBuf, rc::Rc};

type SortedModel<T> = SortModel<Rc<VecModel<T>>, Box<dyn Fn(&T, &T) -> Ordering>>;
type FilteredModel<T> = FilterModel<Rc<SortedModel<T>>, Box<dyn Fn(&T) -> bool>>;
type JustFilteredModel<T> = FilterModel<Rc<VecModel<T>>, Box<dyn Fn(&T) -> bool>>;

#[derive(Clone)]
pub struct AppModel {
    config: Rc<RefCell<Config>>,

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
    pub fn new(config: Config) -> Self {
        let config = Rc::new(RefCell::new(config));

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

        Self {
            config,
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

    pub fn wire(&self, app: &AppWindow) {
        app.set_games(ModelRc::from(self.filtered_games.clone()));
        app.set_homebrew_apps(ModelRc::from(self.filtered_homebrew_apps.clone()));
        app.set_osc_apps(ModelRc::from(self.filtered_osc_apps.clone()));
        app.set_notifications(ModelRc::from(self.notifications.clone()));
        app.set_conversion_queue(ModelRc::from(self.conversion_queue.clone()));
        app.set_conversion_queue_buffer(ModelRc::from(self.conversion_queue_buffer.clone()));
    }

    pub fn set_view_as(&self, view_as: ViewAs) {
        self.config.borrow_mut().contents.view_as = view_as;

        if let Err(e) = self.config.borrow().write() {
            self.notifications.push(e.into());
        }
    }

    pub fn set_sort_by(&self, sort_by: SortBy) {
        self.config.borrow_mut().contents.sort_by = sort_by;

        if let Err(e) = self.config.borrow().write() {
            self.notifications.push(e.into());
        }

        self.sorted_games.reset();
        self.sorted_homebrew_apps.reset();
    }

    pub fn set_show_wii(&self, show_wii: bool) {
        self.config.borrow_mut().contents.show_wii = show_wii;

        if let Err(e) = self.config.borrow().write() {
            self.notifications.push(e.into());
        }

        self.filtered_games.reset();
    }

    pub fn set_show_gc(&self, show_gc: bool) {
        self.config.borrow_mut().contents.show_gc = show_gc;

        if let Err(e) = self.config.borrow().write() {
            self.notifications.push(e.into());
        }

        self.filtered_games.reset();
    }

    pub fn set_mount_point(&self, mount_point: PathBuf) {
        self.config.borrow_mut().contents.mount_point =
            mount_point.to_string_lossy().to_shared_string();

        if let Err(e) = self.config.borrow().write() {
            self.notifications.push(e.into());
        }

        // TODO reload games + apps
    }

    pub fn refresh_all(&self) {
        // TODO reload games + apps
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
