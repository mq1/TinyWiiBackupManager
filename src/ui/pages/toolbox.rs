// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{column, text},
};

pub fn view<'a>(_state: &'a AppState) -> Element<'a, Message> {
    let col = column![text("Toolbox")].padding(10);

    col.into()
}
