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

    // Navigation
    NavTo(Screen),
    UpdateScrollPosition(Id, AbsoluteOffset),

    // Misc
    RefreshGamesAndApps,
    GotWiitdbDatafile(Result<(Datafile, bool), Arc<anyhow::Error>>),
    CloseNotification(usize),
    SelectMountPoint,
    GotDriveUsage(String),
    ChangeTheme,
    UpdateConfig(Config),
    MountPointChosen(Option<PathBuf>),
    OpenThat(OsString),
    AskDeleteDirConfirmation(PathBuf),
    DirectoryDeleted(Result<(), Arc<anyhow::Error>>),
    GotLatestVersion(Result<Option<Version>, Arc<anyhow::Error>>),

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
    Transferred(Result<String, Arc<anyhow::Error>>),
    ChooseArchiveDest(Game),
    ArchiveGame(Option<(Game, PathBuf)>),
    DownloadCoversForUsbLoaderGx,
    DownloadCoversForWiiflow,

    // HBC Apps
    GotHbcAppList(Result<HbcAppList, Arc<anyhow::Error>>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Box<[PathBuf]>),
    HbcAppsInstalled(Result<String, Arc<anyhow::Error>>),
    UpdateHbcFilter(String),

    // OSC Apps
    GotOscAppList(Result<OscAppList, Arc<anyhow::Error>>),
    UpdateOscFilter(String),
    AskInstallOscApp(OscAppMeta),
    InstallOscApp((OscAppMeta, bool)),
    AppInstalled(Result<String, Arc<anyhow::Error>>),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,
}
