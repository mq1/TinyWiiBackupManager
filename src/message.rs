// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    games::{
        disc_info::DiscInfo,
        game::Game,
        game_list::GameList,
        wiitdb::{self},
    },
    hbc::{app_list::HbcAppList, osc::OscAppMeta, osc_list::OscAppList},
    ui::Screen,
    util::DriveInfo,
};
use iced::widget::{Id, operation::AbsoluteOffset};
use semver::Version;
use std::{ffi::OsString, path::PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    // Notification helpers
    GenericResult(Result<String, String>),
    EmptyResult(Result<(), String>),
    GenericError(String),
    GenericSuccess(String),
    CloseNotification(usize),
    CloseAllNotifications,

    // Navigation
    NavTo(Screen),
    UpdateScrollPosition(Id, AbsoluteOffset),

    // Misc
    RefreshGamesAndApps,
    PickMountPoint,
    GotDriveInfo(Option<DriveInfo>),
    ChangeTheme,
    UpdateConfig(Config),
    MountPointPicked(PathBuf),
    OpenThat(OsString),
    AskDeleteDirConfirmation(PathBuf),
    DeleteDirConfirmed(PathBuf),
    GotLatestVersion(Result<Option<Version>, String>),
    FileDropped(PathBuf),
    None,

    // Games
    GotGameList(Result<GameList, String>),
    UpdateGamesFilter(String),
    ShowWii(bool),
    ShowGc(bool),
    PickGames,
    ChooseGamesSrcDir,
    ConfirmAddGamesToTransferStack(Vec<PathBuf>),
    AddGamesToTransferStack(Vec<PathBuf>),
    StartTransfer,
    CancelTransfer(usize),
    GotDiscInfo(Result<DiscInfo, String>),
    SortGamesAndApps(SortBy),
    UpdateTransferStatus(String),
    Transferred(Result<Option<String>, String>),
    ChooseArchiveDest(PathBuf, String),
    ArchiveGame(PathBuf, String, PathBuf),
    DownloadCoversForUsbLoaderGx,
    DownloadCoversForWiiflow,
    DownloadCheatsForGame(Game),
    DownloadCheatsForAllGames,
    DownloadBanners,
    NormalizePaths,
    ConfirmStripGame(Game),
    StripGame(Game),
    ConfirmStripAllGames,
    StripAllGames,
    ChecksumGame(Game),
    ChooseGameToArchiveManually,
    SetManualArchivingGame(PathBuf),
    RunManualGameArchiving,
    GotGameInfo(Result<wiitdb::GameInfo, String>),

    // HBC Apps
    GotHbcAppList(Result<HbcAppList, String>),
    PickHbcApps,
    AddHbcApps(Vec<PathBuf>),
    HbcAppsInstalled(Result<String, String>),
    UpdateHbcFilter(String),
    ChooseFileToWiiload,
    Wiiload(PathBuf),

    // OSC Apps
    GotOscAppList(Result<OscAppList, String>),
    UpdateOscFilter(String),
    AskInstallOscApp(OscAppMeta),
    InstallOscApp(OscAppMeta),
    AppInstalled(Result<String, String>),
    WiiloadOsc(OscAppMeta),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
