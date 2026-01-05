// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{osc::OscApp, ui::Screen, wiitdb};
use iced::font;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
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
    OpenGameDir(usize),
    OpenGameCover(usize),
    GotOscApps(Result<Box<[OscApp]>, String>),
    OpenGameTdb(usize),
    UpdateOscFilter(String),
    FontLoaded(Result<(), font::Error>),
}
