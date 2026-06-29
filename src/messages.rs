// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{notifications::Notification, ui::pages::Page};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum Message {
    NavigateTo(Page),
    RefreshGamesAndApps,
    RefreshPlugins,
    RunTool(u32),
    Notify(Notification),
}
