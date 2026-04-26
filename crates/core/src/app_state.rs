// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config, drive_info::DriveInfo, game::Game, homebrew_app::HomebrewApp, osc::OscAppMeta,
};

#[derive(Debug, Clone)]
pub struct AppState {
    config: Config,
    games: Box<[Game]>,
    homebrew_apps: Box<[HomebrewApp]>,
    osc_apps: Box<[OscAppMeta]>,
    drive_info: DriveInfo,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            games: Box::new([]),
            homebrew_apps: Box::new([]),
            osc_apps: Box::new([]),
            drive_info: DriveInfo::empty(),
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    pub fn homebrew_apps(&self) -> &[HomebrewApp] {
        &self.homebrew_apps
    }

    pub fn homebrew_apps_mut(&mut self) -> &mut [HomebrewApp] {
        &mut self.homebrew_apps
    }

    pub fn osc_apps(&self) -> &[OscAppMeta] {
        &self.osc_apps
    }

    pub fn set_games(&mut self, games: impl Into<Box<[Game]>>) {
        self.games = games.into();
    }

    pub fn set_homebrew_apps(&mut self, homebrew_apps: impl Into<Box<[HomebrewApp]>>) {
        self.homebrew_apps = homebrew_apps.into();
    }

    pub fn set_osc_apps(&mut self, osc_apps: impl Into<Box<[OscAppMeta]>>) {
        self.osc_apps = osc_apps.into();
    }

    pub fn drive_info(&self) -> &DriveInfo {
        &self.drive_info
    }

    pub fn set_drive_info(&mut self, drive_info: DriveInfo) {
        self.drive_info = drive_info;
    }
}
