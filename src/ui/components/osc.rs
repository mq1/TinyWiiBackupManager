// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, message::Message, state::State, ui::components};
use iced::{
    Element, padding,
    widget::{column, row, scrollable, text},
};
use lucide_icons::iced::icon_store;

pub fn view(state: &State) -> Element<'_, Message> {
    let content = match state.config.contents.view_as {
        ViewAs::Grid => components::osc_grid::view(state),
        ViewAs::Table => components::osc_table::view(state),
    };

    if state.osc_filter.is_empty() {
        column![
            components::osc_toolbar::view(state),
            scrollable(column![
                row![
                    icon_store().size(18),
                    text("Open Shop Channel Apps (oscwii.org)").size(18)
                ]
                .spacing(5)
                .padding(padding::left(15).top(10)),
                content
            ])
            .id("osc_scroll")
            .on_scroll(|s| Message::UpdateOscScrollOffset(s.absolute_offset()))
        ]
        .into()
    } else {
        column![components::osc_toolbar::view(state), scrollable(content)].into()
    }
}
