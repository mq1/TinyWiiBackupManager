// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, ViewAs},
    errors::Error,
    games::game_list::GameList,
    homebrew::homebrew_app::HomebrewApp,
    ui::pages::Page,
    util::drive_info::DriveInfo,
};
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(Option<PathBuf>),
    RefreshGamesAndApps,
    CloseNotification(usize),
    GotConfig(Config),
    GotGames(Result<GameList, Error>),
    GotHomebrewApps(Result<Vec<HomebrewApp>, Error>),
    GotDriveInfo(Result<DriveInfo, Error>),
    Open(OsString),
    OpenGameInfo(usize),
    OpenHomebrewAppInfo(usize),
    CloseModal,
    GotDiscInfo(Result<wii_disc_info::Meta, Error>),
    WroteConfig(Result<(), Error>),
    SetViewAs(ViewAs),
    AskDeleteGame(usize),
    AskDeleteHomebrewApp(usize),
    DeleteDir(PathBuf),
    DirDeleted(Result<(), Error>),
}
