// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, message::Message, state::State, ui::components, util::DriveInfo};
use iced::{
    Alignment, Element, Length, padding,
    widget::{column, container, row, scrollable, space, text},
};
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive, icon_waves};

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.config.is_mount_point_valid() {
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

    let content = match state.config.view_as() {
        ViewAs::Grid => components::hbc_grid::view(state),
        ViewAs::Table => components::hbc_table::view(state),
    };

    let usage_str = state
        .drive_info
        .as_ref()
        .map(DriveInfo::get_usage_string)
        .unwrap_or_default();

    if state.hbc_filter.is_empty() {
        column![
            components::hbc_toolbar::view(state),
            scrollable(column![
                row![
                    icon_waves().size(18),
                    text!(
                        "Homebrew Channel Apps: {} found ({})",
                        state.hbc_app_list.count(),
                        state.hbc_app_list.total_size()
                    )
                    .size(18),
                    space::horizontal(),
                    icon_hard_drive(),
                    text(usage_str).size(16),
                ]
                .align_y(Alignment::Center)
                .spacing(5)
                .padding(padding::left(15).right(25).top(10)),
                content
            ])
            .id(state.hbc_scroll_id.clone())
            .spacing(1)
            .on_scroll(|viewport| Message::UpdateScrollPosition(
                state.hbc_scroll_id.clone(),
                viewport.absolute_offset()
            ))
        ]
        .height(Length::Fill)
        .into()
    } else {
        column![components::hbc_toolbar::view(state), scrollable(content)]
            .height(Length::Fill)
            .into()
    }
}
