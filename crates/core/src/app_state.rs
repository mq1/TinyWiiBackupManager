// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config, drive_info::DriveInfo, game::Game, homebrew_app::HomebrewApp, osc::OscAppMeta,
};
use derive_getters::Getters;
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Getters)]
pub struct AppState {
    config: Rc<RefCell<Config>>,
    games: Rc<RefCell<Box<[Game]>>>,
    homebrew_apps: Rc<RefCell<Box<[HomebrewApp]>>>,
    osc_apps: Rc<RefCell<Box<[OscAppMeta]>>>,
    drive_info: Rc<RefCell<DriveInfo>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config: Rc::new(RefCell::new(config)),
            games: Rc::new(RefCell::new(Box::new([]))),
            homebrew_apps: Rc::new(RefCell::new(Box::new([]))),
            osc_apps: Rc::new(RefCell::new(Box::new([]))),
            drive_info: Rc::new(RefCell::new(DriveInfo::empty())),
        }
    }

    pub fn set_config(&self, config: Config) {
        *self.config.borrow_mut() = config;
    }

    pub fn set_games(&self, games: impl Into<Box<[Game]>>) {
        *self.games.borrow_mut() = games.into();
    }

    pub fn set_homebrew_apps(&self, homebrew_apps: impl Into<Box<[HomebrewApp]>>) {
        *self.homebrew_apps.borrow_mut() = homebrew_apps.into();
    }

    pub fn set_osc_apps(&self, osc_apps: impl Into<Box<[OscAppMeta]>>) {
        *self.osc_apps.borrow_mut() = osc_apps.into();
    }

    pub fn set_drive_info(&self, drive_info: DriveInfo) {
        *self.drive_info.borrow_mut() = drive_info;
    }
}
