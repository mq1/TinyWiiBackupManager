// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, messages::Message, state::AppState, ui::components::my_card::my_card};
use iced::{
    Background, Border, Element, Theme, border,
    widget::{button, row, text, tooltip},
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

fn toggle(
    icon: Icon,
    desc: &str,
    variant: ViewAs,
    active: bool,
    is_left: bool,
) -> Element<'_, Message> {
    tooltip(
        button(icon.widget())
            .style(get_button_style(active, is_left))
            .on_press(Message::SetViewAs(variant))
            .padding(8),
        my_card(text(desc)),
        tooltip::Position::Bottom,
    )
    .into()
}

pub fn view_as(state: &AppState) -> Element<'_, Message> {
    my_card(row![
        toggle(
            Icon::Grid,
            "View as grid",
            ViewAs::Grid,
            state.config.view_as == ViewAs::Grid,
            true
        ),
        toggle(
            Icon::Rows3,
            "View as table",
            ViewAs::Table,
            state.config.view_as == ViewAs::Table,
            false
        ),
    ])
    .clip(true)
    .padding(0)
    .into()
}
