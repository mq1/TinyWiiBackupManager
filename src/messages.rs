// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, ViewAs},
    errors::Error,
    games::{game::Game, game_list::GameList},
    homebrew::{homebrew_app::HomebrewApp, homebrew_app_list::HomebrewAppList},
    ui::pages::Page,
    util::drive_info::DriveInfo,
};
use std::{ffi::OsString, path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(Option<PathBuf>),
    RefreshGamesAndApps,
    CloseNotification(usize),
    GotConfig(Config),
    GotGames(Result<GameList, Error>),
    GotHomebrewApps(Result<HomebrewAppList, Error>),
    GotDriveInfo(Result<DriveInfo, Error>),
    Open(OsString),
    OpenGameInfo(Arc<Game>),
    OpenHomebrewAppInfo(Arc<HomebrewApp>),
    CloseModal,
    GotDiscInfo(Result<wii_disc_info::Meta, Error>),
    WroteConfig(Result<(), Error>),
    SetViewAs(ViewAs),
    AskDeleteDir(PathBuf),
    DeleteDir(PathBuf),
    DirDeleted(Result<(), Error>),
    PickHomebrewApps,
    ImportHomebrewApps(Vec<PathBuf>),
    HomebrewAppsImported(Result<usize, Error>),
    SetStatus(String),
    CalcGameSha1(Arc<Game>),
    GotGameSha1(Result<String, Error>),
    PickGames,
    GamesPicked(Vec<PathBuf>),
    ImportGames(Vec<PathBuf>),
    GameImported(Result<(), Error>),
}
