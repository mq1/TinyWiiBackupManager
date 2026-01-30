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
    SelectMountPoint,
    GotDriveInfo(Option<DriveInfo>),
    ChangeTheme,
    UpdateConfig(Config),
    MountPointChosen(Option<PathBuf>),
    OpenThat(OsString),
    AskDeleteDirConfirmation(PathBuf),
    DirectoryDeleted(Result<(), String>),
    GotLatestVersion(Result<Option<Version>, String>),
    FileDropped(PathBuf),

    // Games
    GotGameList(Result<GameList, String>),
    UpdateGamesFilter(String),
    ShowWii(bool),
    ShowGc(bool),
    ChooseGamesToAdd,
    ChooseGamesSrcDir,
    ConfirmAddGamesToTransferStack(Vec<PathBuf>),
    AddGamesToTransferStack((Vec<PathBuf>, bool)),
    StartTransfer,
    CancelTransfer(usize),
    GotDiscInfo(Result<DiscInfo, String>),
    SortGamesAndApps(SortBy),
    UpdateTransferStatus(String),
    Transferred(Result<Option<String>, String>),
    ChooseArchiveDest(Option<PathBuf>, String),
    ArchiveGame(Option<(PathBuf, String, PathBuf)>),
    DownloadCoversForUsbLoaderGx,
    DownloadCoversForWiiflow,
    DownloadCheatsForGame(Game),
    DownloadCheatsForAllGames,
    DownloadBanners,
    NormalizePaths,
    ConfirmStripGame(Game),
    StripGame((Game, bool)),
    ConfirmStripAllGames,
    StripAllGames(bool),
    ChecksumGame(Game),
    ChooseGameToArchiveManually,
    SetManualArchivingGame(Option<PathBuf>),
    RunManualGameArchiving,
    GotGameInfo(Result<wiitdb::GameInfo, String>),

    // HBC Apps
    GotHbcAppList(Result<HbcAppList, String>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Vec<PathBuf>),
    HbcAppsInstalled(Result<String, String>),
    UpdateHbcFilter(String),
    ChooseFileToWiiload,
    Wiiload(Option<PathBuf>),

    // OSC Apps
    GotOscAppList(Result<OscAppList, String>),
    UpdateOscFilter(String),
    AskInstallOscApp(OscAppMeta),
    InstallOscApp((OscAppMeta, bool)),
    AppInstalled(Result<String, String>),
    WiiloadOsc(OscAppMeta),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
