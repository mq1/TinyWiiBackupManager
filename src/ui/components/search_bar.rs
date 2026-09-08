// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Alignment, Border, Element, Theme,
    widget::{row, text_input},
};
use lucide_icons::iced::icon_search;

fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        border: Border::default(),
        ..text_input::default(theme, status)
    }
}

pub fn search_bar<'a>(
    text: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    hint: &'static str,
) -> Element<'a, Message> {
    my_card(
        row![
            icon_search(),
            text_input(hint, text)
                .width(200)
                .on_input(on_input)
                .style(text_input_style),
        ]
        .align_y(Alignment::Center)
        .spacing(5),
    )
    .into()
}
