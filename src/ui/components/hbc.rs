// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, message::Message, state::State, ui::components};
use iced::{
    Element, Length, padding,
    widget::{column, container, row, text},
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

    let mut col = column![components::hbc_toolbar::view(state)];

    if state.hbc_filter.is_empty() {
        col = col.push(
            row![
                icon_waves().size(18),
                text("Homebrew Channel Apps").size(18)
            ]
            .spacing(5)
            .padding(padding::left(20)),
        );
    }

    col = col.push(match state.config.get_view_as() {
        ViewAs::Grid => components::hbc_grid::view(state),
        ViewAs::Table => components::hbc_table::view(state),
    });

    col.into()
}
