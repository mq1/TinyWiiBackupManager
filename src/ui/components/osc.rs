// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{Row, column, row, scrollable, text},
};
use lucide_icons::iced::icon_shopping_bag;

pub fn view(state: &State) -> Element<'_, Message> {
    let mut row = Row::new().width(Length::Fill).spacing(10);

    let filter = state.osc_filter.to_lowercase();
    for (i, app) in state.osc_apps.iter().enumerate() {
        if app.matches_search(&filter) {
            row = row.push(components::osc_card::view(state, i));
        }
    }

    column![
        components::osc_toolbar::view(state),
        scrollable(
            column![
                row![
                    icon_shopping_bag().size(19),
                    text("Open Shop Channel Apps (oscwii.org)").size(18)
                ]
                .spacing(5),
                row.wrap()
            ]
            .spacing(10)
            .padding(10)
        ),
    ]
    .into()
}
