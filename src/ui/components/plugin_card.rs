// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, plugins::plugin::Plugin, ui::components::card};
use iced::{
    Element, Length,
    widget::{row, space, text},
};
use lucide_icons::iced::{icon_check, icon_tag};

pub fn view<'a>(plugin: &'a Plugin) -> Element<'a, Message> {
    card::view(
        row![
            icon_check(),
            text(&plugin.contents.name),
            space().width(10),
            text('|').style(text::secondary),
            space().width(10),
            icon_tag(),
            text(&plugin.contents.version),
        ]
        .spacing(5)
        .width(Length::Fill),
    )
    .into()
}
