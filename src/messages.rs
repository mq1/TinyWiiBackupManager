// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config, games::game::Game, notifications::Notification, plugins::plugin::Plugin,
    ui::pages::Page, util::drive_info::DriveInfo,
};
use serde::Deserialize;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    NoOp,
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(PathBuf),
    RefreshGamesAndApps,
    RefreshPlugins,
    RunTool(usize, usize),
    Notify(Notification),
    CloseNotification(usize),
    GotConfig(Config),
    #[serde(skip)]
    GotGames(Vec<Game>),
    CouldNotGetGames(String),
    GotPlugins(Vec<Plugin>),
    CouldNotGetPlugins(String),
    #[serde(skip)]
    GotDriveInfo(DriveInfo),
    CouldNotGetDriveInfo(String),
    Open(OsString),
    #[serde(skip)]
    OpenGameInfo(usize),
    CloseModal,
    #[serde(skip)]
    GotDiscInfo(Box<wii_disc_info::Meta>),
}
