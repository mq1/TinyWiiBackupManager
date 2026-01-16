// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, message::Message, state::State, ui::components};
use iced::{
    Element, Length,
    widget::{column, container, row, scrollable, text},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive};

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

    let content = match state.config.get_view_as() {
        ViewAs::Grid => components::games_grid::view(state),
        ViewAs::Table => components::games_table::view(state),
    };

    column![
        components::games_toolbar::view(state),
        scrollable(content)
            .id("games_scroll")
            .on_scroll(|s| Message::UpdateGamesScrollOffset(s.absolute_offset()))
    ]
    .into()
}
