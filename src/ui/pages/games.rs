// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element,
    widget::{column, text},
};

pub fn view<'a>() -> Element<'a, Message> {
    column![text("Games")].padding(10).into()
}
