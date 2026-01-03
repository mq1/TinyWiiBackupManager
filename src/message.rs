// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ui::Screen, wiitdb};
use iced::window;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Screen),
    RefreshGames,
    RefreshHbcApps,
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
    GotWindowId(Option<window::Id>),
}
