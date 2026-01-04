// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{Row, column, scrollable, text},
};

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
            column![text("Open Shop Channel").size(18), row.wrap()]
                .spacing(10)
                .padding(10)
        ),
    ]
    .into()
}
