// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{notifications::Notification, ui::pages::Page};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Message {
    NavigateTo(Page),
    RefreshGamesAndApps,
    RefreshPlugins,
    RunTool(usize, usize),
    Notify(Notification),
    MaybeErrored(Option<String>),
}
