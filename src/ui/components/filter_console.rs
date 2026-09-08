// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::my_card::my_card};
use iced::{
    Element, Length,
    widget::{checkbox, container, row, rule, text, tooltip},
};

fn filter<'a, T>(label: &'a str, desc: &'a str, active: bool, on_toggle: T) -> Element<'a, Message>
where
    T: Fn(bool) -> Message + 'a,
{
    tooltip(
        checkbox(active).label(label).on_toggle(on_toggle),
        my_card(text(desc)),
        tooltip::Position::Bottom,
    )
    .into()
}

pub fn filter_console(state: &AppState) -> Element<'_, Message> {
    my_card(
        row![
            container(filter(
                "Wii",
                "Show Wii",
                state.games.filter.show_wii,
                Message::ToggleShowWii
            ))
            .padding(8),
            rule::vertical(1),
            container(filter(
                "GC",
                "Show GameCube",
                state.games.filter.show_ngc,
                Message::ToggleShowNgc
            ))
            .padding(8),
        ]
        .height(Length::Shrink),
    )
    .padding(0)
    .into()
}
