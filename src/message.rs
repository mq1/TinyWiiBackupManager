// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    games::{disc_info::DiscInfo, game_list::GameList, wiitdb::Datafile},
    hbc::{apps::HbcApp, osc::OscAppMeta},
    ui::Screen,
};
use iced::widget::operation::AbsoluteOffset;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    GenericResult(Result<String, String>),
    EmptyResult(Result<(), String>),

    NavigateTo(Screen),
    RefreshGamesAndApps,
    GotWiitdbDatafile(Result<Datafile, String>),
    NotificationTick,
    CloseNotification(usize),
    SelectMountPoint,
    GotDriveUsage(String),
    ChangeTheme,
    UpdateConfig(Config),
    MountPointChosen(Option<PathBuf>),
    OpenThat(OsString),
    AskDeleteDirConfirmation(PathBuf),
    DirectoryDeleted(Result<(), String>),
    UpdateScrollOffset(Screen, AbsoluteOffset),

    // Games
    GotGameList(Result<GameList, String>),
    UpdateGamesFilter(String),
    ShowWii(bool),
    ShowGc(bool),
    ChooseGamesToAdd,
    ChooseGamesSrcDir,
    AddGamesToTransferStack(Vec<PathBuf>),
    StartSingleGameTransfer,
    FinishedTransferringSingleGame(Result<String, String>),
    CancelTransfer(usize),
    GotDiscInfo(usize, Result<DiscInfo, String>),
    SortGamesAndApps(SortBy),
    OpenGameInfo(usize),

    // HBC Apps
    GotHbcApps(Result<Box<[HbcApp]>, String>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Box<[PathBuf]>),
    HbcAppsInstalled(Result<(), String>),
    UpdateHbcFilter(String),

    // OSC Apps
    GotOscApps(Result<Box<[OscAppMeta]>, String>),
    UpdateOscFilter(String),
    AskInstallOscApp(usize),
    InstallOscApp(usize, bool),
    AppInstalled(Result<String, String>),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
