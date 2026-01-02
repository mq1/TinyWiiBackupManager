// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{Element, widget::text};

pub fn view(state: &State) -> Element<'_, Message> {
    text("Open Shop Channel").into()
}
