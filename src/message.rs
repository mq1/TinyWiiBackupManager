// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy},
    games::{disc_info::DiscInfo, game::Game, game_list::GameList, wiitdb::Datafile},
    hbc::{app_list::HbcAppList, osc::OscAppMeta, osc_list::OscAppList},
    ui::Screen,
};
use iced::widget::{Id, operation::AbsoluteOffset};
use semver::Version;
use std::{ffi::OsString, path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub enum Message {
    // Notification helpers
    GenericResult(Result<String, Arc<anyhow::Error>>),
    EmptyResult(Result<(), Arc<anyhow::Error>>),
    GenericError(Arc<anyhow::Error>),
    GenericSuccess(String),
    CloseNotification(usize),
    CloseAllNotifications,

    // Navigation
    NavTo(Screen),
    UpdateScrollPosition(Id, AbsoluteOffset),

    // Misc
    RefreshGamesAndApps,
    GotWiitdbDatafile(Result<(Datafile, bool), Arc<anyhow::Error>>),
    SelectMountPoint,
    GotDriveUsage(String),
    ChangeTheme,
    UpdateConfig(Config),
    MountPointChosen(Option<PathBuf>),
    OpenThat(OsString),
    AskDeleteDirConfirmation(PathBuf),
    DirectoryDeleted(Result<(), Arc<anyhow::Error>>),
    GotLatestVersion(Result<Option<Version>, Arc<anyhow::Error>>),
    FileDropped(PathBuf),

    // Games
    GotGameList(Result<GameList, Arc<anyhow::Error>>),
    UpdateGamesFilter(String),
    ShowWii(bool),
    ShowGc(bool),
    ChooseGamesToAdd,
    ChooseGamesSrcDir,
    ConfirmAddGamesToTransferStack(Vec<PathBuf>),
    AddGamesToTransferStack((Vec<PathBuf>, bool)),
    StartTransfer,
    CancelTransfer(usize),
    GotDiscInfo(Result<DiscInfo, Arc<anyhow::Error>>),
    SortGamesAndApps(SortBy),
    UpdateTransferStatus(String),
    Transferred(Result<Option<String>, Arc<anyhow::Error>>),
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

    // HBC Apps
    GotHbcAppList(Result<HbcAppList, Arc<anyhow::Error>>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Vec<PathBuf>),
    HbcAppsInstalled(Result<String, Arc<anyhow::Error>>),
    UpdateHbcFilter(String),
    ChooseFileToWiiload,
    Wiiload(Option<PathBuf>),

    // OSC Apps
    GotOscAppList(Result<OscAppList, Arc<anyhow::Error>>),
    UpdateOscFilter(String),
    AskInstallOscApp(OscAppMeta),
    InstallOscApp((OscAppMeta, bool)),
    AppInstalled(Result<String, Arc<anyhow::Error>>),
    WiiloadOsc(OscAppMeta),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
