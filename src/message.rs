// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{SortBy, ViewAs},
    disc_info::DiscInfo,
    game::Game,
    hbc::HbcApp,
    osc::OscAppMeta,
    ui::Screen,
    wiitdb,
};
use iced::widget::operation::AbsoluteOffset;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    GenericResult(Result<String, String>),
    EmptyResult(Result<(), String>),

    NavigateTo(Screen),
    RefreshGamesAndApps,
    OpenProjectRepo,
    GotWiitdbDatafile(Result<wiitdb::Datafile, String>),
    NotificationTick,
    CloseNotification(usize),
    SelectMountPoint,
    MountPointChosen(Option<PathBuf>),
    GotDriveUsage(String),
    ChangeTheme,
    OpenDataDir,
    UpdateGamesScrollOffset(AbsoluteOffset),
    UpdateHbcScrollOffset(AbsoluteOffset),
    UpdateOscScrollOffset(AbsoluteOffset),

    // Games
    GotGames(Result<Box<[Game]>, String>),
    UpdateGamesFilter(String),
    ShowWii(bool),
    ShowGc(bool),
    AskDeleteGame(usize),
    DeleteGame(usize, bool),
    GameDeleted(Result<String, String>),
    OpenGameDir(usize),
    OpenGameTdb(usize),
    ChooseGamesToAdd,
    ChooseGamesSrcDir,
    AddGamesToTransferStack(Vec<PathBuf>),
    StartSingleGameTransfer,
    FinishedTransferringSingleGame(Result<String, String>),
    CancelTransfer(usize),
    GotDiscInfo(usize, Result<DiscInfo, String>),

    // HBC Apps
    GotHbcApps(Result<Box<[HbcApp]>, String>),
    ChooseHbcAppsToAdd,
    AddHbcApps(Box<[PathBuf]>),
    HbcAppsInstalled(Result<(), String>),
    UpdateHbcFilter(String),
    OpenHbcPage(usize),
    AskDeleteHbcApp(usize),
    DeleteHbcApp(usize, bool),
    AppDeleted(Result<String, String>),

    // OSC Apps
    GotOscApps(Result<Box<[OscAppMeta]>, String>),
    UpdateOscFilter(String),
    AskInstallOscApp(usize),
    InstallOscApp(usize, bool),
    AppInstalled(Result<String, String>),
    OpenOscPage(usize),

    // Toolbox
    DownloadWiitdbToDrive,
    #[cfg(target_os = "macos")]
    RunDotClean,

    // Settings
    UpdateWiiOutputFormat(nod::common::Format),
    UpdateSortBy(SortBy),
    UpdateViewAs(ViewAs),
}
