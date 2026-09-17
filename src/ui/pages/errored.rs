// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length,
    widget::{container, text},
};

pub fn errored<'a>(e: impl std::fmt::Display + 'a) -> Element<'a, Message> {
    container(text!("Error: {e}")).center(Length::Fill).into()
}
