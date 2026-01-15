// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, message::Message, state::State, ui::components};
use iced::{
    Alignment, Element, Length, padding,
    widget::{column, container, row, scrollable, space, text},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive, icon_waves};
use size::Size;

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
        ViewAs::Grid => components::hbc_grid::view(state),
        ViewAs::Table => components::hbc_table::view(state),
    };

    if state.hbc_filter.is_empty() {
        let size = state
            .hbc_apps
            .iter()
            .map(|app| app.size)
            .fold(Size::from_bytes(0), |a, b| a + b);

        column![
            components::hbc_toolbar::view(state),
            scrollable(column![
                row![
                    icon_waves().size(18),
                    text!("Homebrew Channel Apps ({})", size).size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(&state.drive_usage).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                content
            ])
        ]
        .into()
    } else {
        column![components::hbc_toolbar::view(state), scrollable(content)].into()
    }
}
