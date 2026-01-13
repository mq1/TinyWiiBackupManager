// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy,
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Element,
    widget::{button, container, row},
};
use lucide_icons::iced::{
    icon_arrow_down_a_z, icon_arrow_down_narrow_wide, icon_arrow_up_a_z, icon_arrow_up_narrow_wide,
};

pub fn view(state: &State) -> Element<'_, Message> {
    let sort_by = state.config.get_sort_by();

    let sort_by_name_button = match sort_by {
        SortBy::NameAscending => button(icon_arrow_down_a_z())
            .on_press(Message::UpdateSortBy(SortBy::NameDescending))
            .style(style::active_nav_button),
        SortBy::NameDescending => button(icon_arrow_up_a_z())
            .on_press(Message::UpdateSortBy(SortBy::NameAscending))
            .style(style::active_nav_button),
        _ => button(icon_arrow_down_a_z())
            .on_press(Message::UpdateSortBy(SortBy::NameAscending))
            .style(style::inactive_nav_button),
    };

    let sort_by_size_button = match sort_by {
        SortBy::SizeAscending => button(icon_arrow_down_narrow_wide())
            .on_press(Message::UpdateSortBy(SortBy::SizeDescending))
            .style(style::active_nav_button),
        SortBy::SizeDescending => button(icon_arrow_up_narrow_wide())
            .on_press(Message::UpdateSortBy(SortBy::SizeAscending))
            .style(style::active_nav_button),
        _ => button(icon_arrow_down_narrow_wide())
            .on_press(Message::UpdateSortBy(SortBy::SizeAscending))
            .style(style::inactive_nav_button),
    };

    container(
        row![
            components::my_tooltip::view(sort_by_name_button, "Sort by Name"),
            components::my_tooltip::view(sort_by_size_button, "Sort by Size"),
        ]
        .spacing(5)
        .padding(5),
    )
    .style(style::card)
    .into()
}
