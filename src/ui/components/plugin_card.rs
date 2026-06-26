// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, plugins::plugin::Plugin, ui::components::card};
use iced::{
    Element, Length,
    widget::{row, text},
};
use lucide_icons::iced::{icon_check, icon_tag};

pub fn view<'a>(plugin: &'a Plugin) -> Element<'a, Message> {
    card::view(
        row![
            icon_check(),
            text(&plugin.name),
            text("  |  ").style(text::secondary),
            icon_tag(),
            text(&plugin.version),
        ]
        .spacing(5)
        .width(Length::Fill),
    )
    .into()
}
