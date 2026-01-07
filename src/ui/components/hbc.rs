// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{Row, column, container, row, scrollable, text},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive, icon_waves};

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.config.valid_mount_point() {
        return container(
            row![
                icon_arrow_down_left(),
                text("Click on"),
                icon_hard_drive(),
                text("to select a Drive/Mount Point")
            ]
            .spacing(5),
        )
        .center(Length::Fill)
        .into();
    }

    let mut row = Row::new().width(Length::Fill).spacing(10);

    let filter = state.hbc_filter.to_lowercase();
    for (i, app) in state.hbc_apps.iter().enumerate() {
        if app.matches_search(&filter) {
            row = row.push(components::hbc_card::view(state, i));
        }
    }

    column![
        components::hbc_toolbar::view(state),
        scrollable(
            column![
                row![
                    icon_waves().size(18),
                    text("Homebrew Channel Apps").size(18)
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
