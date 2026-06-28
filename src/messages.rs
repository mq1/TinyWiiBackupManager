// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ui::pages::Page;

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Page),
    RefreshGamesAndApps,
    RefreshPlugins,
    RunLuaFunction(Vec<u8>),
}
