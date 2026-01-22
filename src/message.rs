// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    games::{disc_info::DiscInfo, game_list::GameList, wiitdb::Datafile},
    hbc::{app_list::HbcAppList, osc::OscAppMeta, osc_list::OscAppList},
};
use iced::widget::scrollable::Viewport;
use semver::Version;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    // Notification helpers
    GenericResult(Result<String, String>),
    EmptyResult(Result<(), String>),

    // Navigation
    NavToGames,
    NavToGameInfo(PathBuf),
    NavToHbcApps,
    NavToHbcAppInfo(PathBuf),
    NavToOscApps,
    NavToOscAppInfo(String),
    NavToToolbox,
    NavToTransfer,
    NavToSettings,
    NavToAbout,
    UpdateGamesScrollOffset(Viewport),
    UpdateHbcScrollOffset(Viewport),
    UpdateOscScrollOffset(Viewport),

    // Misc
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
    GotLatestVersion(Result<Option<Version>, String>),

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
    GotDiscInfo(Result<DiscInfo, String>),
    SortGamesAndApps(SortBy),

    // HBC Apps
    GotHbcAppList(Result<HbcAppList, String>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Box<[PathBuf]>),
    HbcAppsInstalled(Result<(), String>),
    UpdateHbcFilter(String),

    // OSC Apps
    GotOscAppList(Result<OscAppList, String>),
    UpdateOscFilter(String),
    AskInstallOscApp(OscAppMeta),
    InstallOscApp((OscAppMeta, bool)),
    AppInstalled(Result<String, String>),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
