// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    DisplayedGame, DisplayedHomebrewApp, DisplayedOscApp, Notification, QueuedConversion, games,
    homebrew_apps, osc,
};
use slint::{FilterModel, SortModel, VecModel};
use std::{cell::RefCell, rc::Rc};
use twbm_core::{
    config::{Config, SortBy},
    drive_info::DriveInfo,
    game::Game,
    homebrew_app::HomebrewApp,
    osc::OscAppMeta,
};

#[allow(clippy::type_complexity)]
pub struct State<SG, SH, FG, FH, FO>
where
    SG: FnMut(&DisplayedGame, &DisplayedGame) -> std::cmp::Ordering + 'static,
    SH: FnMut(&DisplayedHomebrewApp, &DisplayedHomebrewApp) -> std::cmp::Ordering + 'static,
    FG: Fn(&DisplayedGame) -> bool + 'static,
    FH: Fn(&DisplayedHomebrewApp) -> bool + 'static,
    FO: Fn(&DisplayedOscApp) -> bool + 'static,
{
    pub config: Config,
    pub sort_by: Rc<RefCell<SortBy>>,
    pub show_wii: Rc<RefCell<bool>>,
    pub show_gc: Rc<RefCell<bool>>,
    pub games: Vec<Game>,
    pub homebrew_apps: Vec<HomebrewApp>,
    pub osc_apps: Vec<OscAppMeta>,
    pub drive_info: DriveInfo,
    pub games_filter: Rc<RefCell<String>>,
    pub homebrew_apps_filter: Rc<RefCell<String>>,
    pub osc_apps_filter: Rc<RefCell<String>>,
    pub displayed_games: Rc<VecModel<DisplayedGame>>,
    pub displayed_homebrew_apps: Rc<VecModel<DisplayedHomebrewApp>>,
    pub displayed_osc_apps: Rc<VecModel<DisplayedOscApp>>,
    pub sorted_games: Rc<SortModel<Rc<VecModel<DisplayedGame>>, SG>>,
    pub sorted_homebrew_apps: Rc<SortModel<Rc<VecModel<DisplayedHomebrewApp>>, SH>>,
    pub filtered_games: Rc<FilterModel<Rc<SortModel<Rc<VecModel<DisplayedGame>>, SG>>, FG>>,
    pub filtered_homebrew_apps:
        Rc<FilterModel<Rc<SortModel<Rc<VecModel<DisplayedHomebrewApp>>, SH>>, FH>>,
    pub filtered_osc_apps: Rc<FilterModel<Rc<VecModel<DisplayedOscApp>>, FO>>,
    pub conversion_queue: Rc<VecModel<QueuedConversion>>,
    pub conversion_queue_buffer: Rc<VecModel<QueuedConversion>>,
    pub notifications: Rc<VecModel<Notification>>,
    pub is_converting: bool,
    pub is_downloading_osc_icons: bool,
    pub is_downloading_covers: bool,
}

#[allow(clippy::type_complexity)]
pub fn new_state() -> State<
    impl FnMut(&DisplayedGame, &DisplayedGame) -> std::cmp::Ordering + 'static,
    impl FnMut(&DisplayedHomebrewApp, &DisplayedHomebrewApp) -> std::cmp::Ordering + 'static,
    impl Fn(&DisplayedGame) -> bool + 'static,
    impl Fn(&DisplayedHomebrewApp) -> bool + 'static,
    impl Fn(&DisplayedOscApp) -> bool + 'static,
> {
    let config = Config::load();

    let sort_by = Rc::new(RefCell::new(config.contents.sort_by));
    let show_wii = Rc::new(RefCell::new(config.contents.show_wii));
    let show_gc = Rc::new(RefCell::new(config.contents.show_gc));

    let games = Vec::<Game>::new();
    let homebrew_apps = Vec::<HomebrewApp>::new();
    let osc_apps = Vec::<OscAppMeta>::new();
    let drive_info = DriveInfo::empty();

    let displayed_games = Rc::new(VecModel::from(Vec::new()));
    let games_filter = Rc::new(RefCell::new(String::new()));
    let sorted_games = Rc::new(SortModel::new(
        displayed_games.clone(),
        games::get_compare_fn(sort_by.clone()),
    ));
    let filtered_games = Rc::new(FilterModel::new(
        sorted_games.clone(),
        games::get_filter_fn(games_filter.clone(), show_wii.clone(), show_gc.clone()),
    ));

    let displayed_homebrew_apps = Rc::new(VecModel::from(Vec::new()));
    let homebrew_apps_filter = Rc::new(RefCell::new(String::new()));
    let sorted_homebrew_apps = Rc::new(SortModel::new(
        displayed_homebrew_apps.clone(),
        homebrew_apps::get_compare_fn(sort_by.clone()),
    ));
    let filtered_homebrew_apps = Rc::new(FilterModel::new(
        sorted_homebrew_apps.clone(),
        homebrew_apps::get_filter_fn(homebrew_apps_filter.clone()),
    ));

    let displayed_osc_apps = Rc::new(VecModel::from(Vec::new()));
    let osc_apps_filter = Rc::new(RefCell::new(String::new()));
    let filtered_osc_apps = Rc::new(FilterModel::new(
        displayed_osc_apps.clone(),
        osc::get_filter_fn(osc_apps_filter.clone()),
    ));

    let conversion_queue = Rc::new(VecModel::from(Vec::new()));
    let conversion_queue_buffer = Rc::new(VecModel::from(Vec::new()));
    let notifications = Rc::new(VecModel::from(Vec::new()));

    State {
        config,
        sort_by,
        show_wii,
        show_gc,
        games,
        homebrew_apps,
        osc_apps,
        drive_info,
        displayed_games,
        games_filter,
        sorted_games,
        filtered_games,
        displayed_homebrew_apps,
        homebrew_apps_filter,
        sorted_homebrew_apps,
        filtered_homebrew_apps,
        displayed_osc_apps,
        osc_apps_filter,
        filtered_osc_apps,
        conversion_queue,
        conversion_queue_buffer,
        notifications,
        is_converting: false,
        is_downloading_osc_icons: false,
        is_downloading_covers: false,
    }
}
