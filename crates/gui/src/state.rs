// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DisplayedGame, DisplayedHomebrewApp, DisplayedOscApp, Notification};
use slint::{SharedString, VecModel};
use std::{collections::VecDeque, rc::Rc};
use twbm_core::{
    config::Config, conversion_queue::QueuedConversion, drive_info::DriveInfo, game::Game,
    homebrew_app::HomebrewApp, osc::OscApp,
};

pub struct State {
    pub config: Config,
    pub games: Vec<Game>,
    pub homebrew_apps: Vec<HomebrewApp>,
    pub osc_apps: Vec<OscApp>,
    pub drive_info: DriveInfo,
    pub displayed_games: Rc<VecModel<DisplayedGame>>,
    pub displayed_homebrew_apps: Rc<VecModel<DisplayedHomebrewApp>>,
    pub displayed_osc_apps: Rc<VecModel<DisplayedOscApp>>,
    pub conversion_queue: VecDeque<QueuedConversion>,
    pub displayed_conversion_queue: Rc<VecModel<SharedString>>,
    pub games_to_add: Rc<VecModel<SharedString>>,
    pub notifications: Rc<VecModel<Notification>>,
    pub is_converting: bool,
    pub is_downloading_osc_icons: bool,
    pub is_downloading_covers: bool,
    pub games_filter: String,
    pub homebrew_apps_filter: String,
    pub osc_apps_filter: String,
}

impl State {
    pub fn new() -> Self {
        State {
            config: Config::load(),
            games: Vec::new(),
            homebrew_apps: Vec::new(),
            osc_apps: Vec::new(),
            drive_info: DriveInfo::default(),
            displayed_games: Rc::new(VecModel::from(Vec::new())),
            displayed_homebrew_apps: Rc::new(VecModel::from(Vec::new())),
            displayed_osc_apps: Rc::new(VecModel::from(Vec::new())),
            conversion_queue: VecDeque::new(),
            displayed_conversion_queue: Rc::new(VecModel::from(Vec::new())),
            games_to_add: Rc::new(VecModel::from(Vec::new())),
            notifications: Rc::new(VecModel::from(Vec::new())),
            is_converting: false,
            is_downloading_osc_icons: false,
            is_downloading_covers: false,
            games_filter: String::new(),
            homebrew_apps_filter: String::new(),
            osc_apps_filter: String::new(),
        }
    }
}
