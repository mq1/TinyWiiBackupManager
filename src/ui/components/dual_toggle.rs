// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use derive_setters::Setters;
use iced::{
    Background, Border, Element, Theme, border,
    widget::{Row, button, text, tooltip},
};
use lucide_icons::Icon;

fn get_border(is_left: bool) -> Border {
    border::rounded(match is_left {
        true => border::left(10),
        false => border::right(10),
    })
}

fn get_button_style(
    active: bool,
    is_left: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |theme, status| button::Style {
        background: active
            .then_some(theme.palette().primary.scale_alpha(0.5))
            .map(Background::Color),
        border: get_border(is_left),
        ..button::text(theme, status)
    }
}

fn toggle((index, item): (usize, DualToggleItem<'_>)) -> Element<'_, Message> {
    tooltip(
        button(item.icon.widget())
            .style(get_button_style(item.active, index == 0))
            .on_press(item.on_press)
            .padding(8),
        my_card(text(item.desc)),
        tooltip::Position::Bottom,
    )
    .into()
}

#[derive(Setters)]
pub struct DualToggleItem<'a> {
    pub icon: Icon,
    pub desc: &'a str,
    pub active: bool,
    pub on_press: Message,
}

pub fn dual_toggle(items: [DualToggleItem<'_>; 2]) -> Element<'_, Message> {
    my_card(
        items
            .into_iter()
            .enumerate()
            .map(toggle)
            .collect::<Row<'_, _>>(),
    )
    .padding(0)
    .into()
}
