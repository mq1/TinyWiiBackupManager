// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy,
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Alignment, Element,
    widget::{button, container, row, space, text_input},
};
use lucide_icons::iced::{
    icon_arrow_down_a_z, icon_arrow_down_narrow_wide, icon_arrow_up_a_z, icon_arrow_up_narrow_wide,
    icon_plus, icon_search,
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

    row![
        container(
            row![
                space(),
                icon_search(),
                text_input("Search by Name", &state.hbc_filter)
                    .width(200)
                    .style(style::search_bar)
                    .on_input(Message::UpdateHbcFilter),
            ]
            .spacing(10)
            .padding(5)
            .align_y(Alignment::Center)
        )
        .style(style::card),
        space::horizontal(),
        container(
            row![
                components::my_tooltip::view(sort_by_name_button, "Sort by Name"),
                components::my_tooltip::view(sort_by_size_button, "Sort by Size"),
            ]
            .spacing(5)
            .padding(5)
        )
        .style(style::card),
        space(),
        space(),
        button(icon_plus().size(18).center())
            .width(35)
            .height(35)
            .style(style::rounded_button)
    ]
    .spacing(5)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
