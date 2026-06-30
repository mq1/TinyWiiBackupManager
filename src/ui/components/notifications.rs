// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{Column, text},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    state
        .notifications
        .iter()
        .map(|notification| text(&notification.label).into())
        .collect::<Column<_>>()
        .into()
}
