// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element,
    widget::{rich_text, span},
};
use std::{borrow::Cow, ffi::OsStr};

pub fn view<'a>(label: impl Into<Cow<'a, str>>, url: &'a OsStr) -> Element<'a, Message> {
    rich_text![span(label.into()).link(url)]
        .on_link_click(Message::Open)
        .into()
}
