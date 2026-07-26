// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config, errors::Error, games::game::Game, ui::pages::Page, util::drive_info::DriveInfo,
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    NoOp,
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(PathBuf),
    RefreshGamesAndApps,
    CloseNotification(usize),
    GotConfig(Config),
    GotGames(Result<Vec<Game>, Error>),
    GotDriveInfo(Result<DriveInfo, Error>),
    Open(OsString),
    OpenGameInfo(usize),
    CloseModal,
    GotDiscInfo(Result<Box<wii_disc_info::Meta>, Error>),
    WroteConfig(Result<(), Error>),
}
