// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config, games::game::Game, notifications::Notification, ui::pages::Page,
    util::drive_info::DriveInfo,
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(PathBuf),
    RefreshGamesAndApps,
    Notify(Notification),
    CloseNotification(usize),
    GotConfig(Config),
    GotGames(Vec<Game>),
    CouldNotGetGames(String),
    GotDriveInfo(DriveInfo),
    CouldNotGetDriveInfo(String),
    Open(OsString),
    OpenGameInfo(usize),
    CloseModal,
    GotDiscInfo(Box<wii_disc_info::Meta>),
}
