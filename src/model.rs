// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    Config, DriveInfo, Game, GcOutputFormat, HomebrewApp, Notification, OscApp, QueuedConversion,
    SortBy, ThemePreference, TxtCodesSource, ViewAs, WiiOutputFormat, convert::Conversion, game,
    homebrew_app, mirrored::Mirrored, osc,
};
use slint::{FilterModel, Model, SharedString, SortModel, ToSharedString, VecModel};
use std::{cell::RefCell, cmp::Ordering, path::PathBuf, rc::Rc};

type SortedModel<T> = SortModel<Rc<VecModel<T>>, Box<dyn Fn(&T, &T) -> Ordering>>;
type FilteredModel<T> = FilterModel<Rc<SortedModel<T>>, Box<dyn Fn(&T) -> bool>>;
type JustFilteredModel<T> = FilterModel<Rc<VecModel<T>>, Box<dyn Fn(&T) -> bool>>;

#[derive(Clone)]
pub struct AppModel {
    config: Rc<Mirrored<Config>>,
    drive_info: Rc<Mirrored<DriveInfo>>,
    status: Rc<Mirrored<SharedString>>,
    crc32_status: Rc<Mirrored<SharedString>>,

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

    is_converting: Rc<RefCell<bool>>,
}

impl AppModel {
    pub fn new(config: Config) -> Self {
        let config = Rc::new(Mirrored::new(config));
        let drive_info = Rc::new(Mirrored::new(DriveInfo::default()));

        let status = Rc::new(Mirrored::new(SharedString::new()));
        let crc32_status = Rc::new(Mirrored::new(SharedString::new()));

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
            status,
            crc32_status,
            drive_info,
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
            is_converting: Rc::new(RefCell::new(false)),
        }
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

    pub fn set_conversion_queue_buffer(&self, new: Vec<QueuedConversion>) {
        self.conversion_queue_buffer.set_vec(new);
    }

    pub fn confirm_conversion_queue_buffer(&self) {
        self.conversion_queue
            .extend(self.conversion_queue_buffer.iter());
        self.conversion_queue_buffer.clear();
    }

    pub fn clear_conversion_queue_buffer(&self) {
        self.conversion_queue_buffer.clear();
    }

    pub fn remove_queued_conversion(&self, index: usize) {
        self.conversion_queue.remove(index);
    }

    pub fn clear_conversion_queue(&self) {
        self.conversion_queue.clear();
    }

    pub fn set_drive_info(&self, drive_info: DriveInfo) {
        self.drive_info.set(drive_info);
    }

    pub fn set_is_converting(&self, is_converting: bool) {
        *self.is_converting.borrow_mut() = is_converting;
    }

    pub fn pop_conversion(&self) -> Option<Conversion> {
        if self.conversion_queue.row_count() == 0 {
            return None;
        }

        let queued = self.conversion_queue.remove(0);
        let conf = &self.config.borrow().contents;
        let drive_info = self.drive_info.borrow();

        let conv = Conversion::new(&queued, conf, &drive_info);
        Some(conv)
    }

    pub fn set_status(&self, status: SharedString) {
        self.status.set(status);
    }

    pub fn set_crc32_status(&self, status: SharedString) {
        self.crc32_status.set(status);
    }
}

// Getters
impl AppModel {
    pub fn config(&self) -> &Mirrored<Config> {
        &self.config
    }

    pub fn drive_info(&self) -> &Mirrored<DriveInfo> {
        &self.drive_info
    }

    pub fn games(&self) -> Rc<FilteredModel<Game>> {
        self.filtered_games.clone()
    }

    pub fn homebrew_apps(&self) -> Rc<FilteredModel<HomebrewApp>> {
        self.filtered_homebrew_apps.clone()
    }

    pub fn osc_apps(&self) -> Rc<JustFilteredModel<OscApp>> {
        self.filtered_osc_apps.clone()
    }

    pub fn notifications(&self) -> Rc<VecModel<Notification>> {
        self.notifications.clone()
    }

    pub fn conversion_queue(&self) -> Rc<VecModel<QueuedConversion>> {
        self.conversion_queue.clone()
    }

    pub fn conversion_queue_buffer(&self) -> Rc<VecModel<QueuedConversion>> {
        self.conversion_queue_buffer.clone()
    }

    pub fn existing_ids(&self) -> Vec<String> {
        self.games.iter().map(|g| g.id.to_string()).collect()
    }

    pub fn is_converting(&self) -> bool {
        *self.is_converting.borrow()
    }

    pub fn status(&self) -> &Mirrored<SharedString> {
        &self.status
    }

    pub fn crc32_status(&self) -> &Mirrored<SharedString> {
        &self.crc32_status
    }
}
