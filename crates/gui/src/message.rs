// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, message::ConfigMessage},
    long_operation::message::LongOperationsMessage,
    notifications::notification::Notification,
    osc::{osc_app::OscApp, osc_state::OscAppList},
    state::LazyState,
    toolbox::ToolboxItem,
    ui::pages::Page,
};

use semver::Version;
use std::{ffi::OsString, path::PathBuf};
use twbm_driveinfo::DriveInfoState;
use twbm_games::{game::Game, import_entry::ImportEntry, manager::GameManager};
use twbm_homebrew::{homebrew_app::HomebrewApp, homebrew_app_manager::HomebrewAppManager};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub enum Message {
    Config(ConfigMessage),
    LongOperations(LongOperationsMessage),

    NoOp,
    NavigateTo(Page),
    PickMountPoint,
    MountPointPicked(PathBuf),
    RefreshGamesAndApps,
    Notify(Notification),
    CloseNotification(usize),
    GotConfig(Config),
    GotGames(GameManager),
    GotHomebrew(HomebrewAppManager),
    GotDrive(DriveInfoState),
    Open(OsString),
    OpenGameInfo(Game),
    OpenHomebrewAppInfo(Box<HomebrewApp>),
    CloseModal,
    GotDiscInfo(wii_disc_info::Meta),
    AskDeleteDir(PathBuf),
    DeleteDir(PathBuf),
    DirDeleted(Result<(), String>),
    PickHomebrewApps,
    ImportHomebrewApps(Vec<PathBuf>),
    HomebrewAppsImported(usize),
    CalcGameSha1(Game),
    PickGames,
    PickGamesRecursively,
    PickedGames(Vec<ImportEntry>),
    CancelImport(usize),
    CancelAllImports,
    ToggleAnimationState,
    ReloadCover(GameID),
    ReloadOscIcon(usize),
    SearchGames(String),
    ToggleShowWii(bool),
    ToggleShowNgc(bool),
    SearchHomebrewApps(String),
    PickExportDest(Game),
    ExportGame(Game, PathBuf),
    RunTool(&'static ToolboxItem),
    PickFileToSendViaWiiload,
    SendViaWiiload(PathBuf),
    SendOscAppViaWiiload(OscApp),
    RefreshOscContents,
    GotOscContents(LazyState<OscAppList>),
    SearchOscApps(String),
    OpenOscAppInfo(OscApp),
    InstallOscApp(OscApp),
    OscAppInstalled(Result<Box<str>, String>),
    DownloadTxtCodes(Game),
    PickGameToConvert,
    ConvertGame(PathBuf, PathBuf),
    GotUpdate(Version),
    TriggerImport,
    FileDropped(PathBuf),
    GotUpdatePartitionCheckResult(bool),
    AskConfirmScrub(Game),
    Scrub(Game),
}
