// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ui::Screen, wiitdb};

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
}
