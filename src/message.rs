// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::SortBy, game::Game, hbc::HbcApp, osc::OscAppMeta, ui::Screen, wiitdb};
use iced::font;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    EmptyResult(Result<(), String>),

    NavigateTo(Screen),
    RefreshGamesAndApps,
    OpenProjectRepo,
    UpdateGamesFilter(String),
    GotWiitdbDatafile(Result<wiitdb::Datafile, String>),
    NotificationTick,
    CloseNotification(usize),
    ShowWii(bool),
    ShowGc(bool),
    SelectMountPoint,
    MountPointChosen(Option<PathBuf>),
    AskDeleteGame(usize),
    DeleteGame(usize, bool),
    GameDeleted(Result<String, String>),
    OpenGameDir(usize),
    GotOscApps(Result<Box<[OscAppMeta]>, String>),
    OpenGameTdb(usize),
    UpdateOscFilter(String),
    FontLoaded(Result<(), font::Error>),
    OpenOscPage(usize),
    GotGames(Result<Box<[Game]>, String>),
    GotHbcApps(Result<Box<[HbcApp]>, String>),
    GotDriveUsage(String),
    AskInstallOscApp(usize),
    InstallOscApp(usize, bool),
    AppInstalled(Result<String, String>),
    UpdateHbcFilter(String),
    ChangeTheme,
    AskDeleteHbcApp(usize),
    DeleteHbcApp(usize, bool),
    AppDeleted(Result<String, String>),
    OpenHbcPage(usize),
    OpenDataDir,
    DownloadWiitdbToDrive,
    FinishedDownloadingWiitdbToDrive(Result<(), String>),

    // Settings
    UpdateWiiOutputFormat(nod::common::Format),
    UpdateSortBy(SortBy),
}
