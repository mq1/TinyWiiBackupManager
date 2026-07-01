// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game, notifications::Notification, plugins::plugin::Plugin, ui::pages::Page,
    util::drive_info::DriveInfo,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    NoOp,
    NavigateTo(Page),
    RefreshGamesAndApps,
    RefreshPlugins,
    RunTool(usize, usize),
    Notify(Notification),
    CloseNotification(usize),
    GotGames(Vec<Game>),
    CouldNotGetGames(String),
    GotPlugins(Vec<Plugin>),
    CouldNotGetPlugins(String),
    GotDriveInfo(DriveInfo),
    CouldNotGetDriveInfo(String),
}
