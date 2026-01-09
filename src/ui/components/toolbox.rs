// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Element, Length,
    widget::{column, row, text},
};
use lucide_icons::iced::icon_tool_case;

pub fn view(state: &State) -> Element<'_, Message> {
    column![row![icon_tool_case().size(18), text("Toolbox").size(18)].spacing(5)]
        .spacing(10)
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
