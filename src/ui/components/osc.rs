// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length, padding,
    widget::{Row, column, row, scrollable, text},
};
use lucide_icons::iced::icon_store;

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.osc_filter.is_empty() {
        let mut row = Row::new().width(Length::Fill).spacing(10).padding(10);

        for i in state.filtered_osc_indices.iter().copied() {
            row = row.push(components::osc_card::view(state, i));
        }

        return column![components::osc_toolbar::view(state), scrollable(row.wrap())].into();
    }

    let mut row = Row::new().width(Length::Fill).spacing(10).padding(10);
    for i in 0..state.osc_apps.len() {
        row = row.push(components::osc_card::view(state, i));
    }

    column![
        components::osc_toolbar::view(state),
        scrollable(column![
            row![
                icon_store().size(18),
                text("Open Shop Channel Apps (oscwii.org)").size(18)
            ]
            .spacing(5)
            .padding(padding::left(15)),
            row.wrap()
        ]),
    ]
    .spacing(10)
    .into()
}
