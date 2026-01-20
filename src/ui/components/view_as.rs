// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    message::Message,
    state::State,
    ui::{components, style},
};
use iced::{
    Element,
    widget::{button, container, row},
};
use lucide_icons::iced::{icon_layout_grid, icon_table};

pub fn view(state: &State) -> Element<'_, Message> {
    let view_as = state.config.view_as();

    let view_as_grid_button = button(icon_layout_grid())
        .style(style::get_nav_button_style(view_as == ViewAs::Grid))
        .on_press_with(|| Message::UpdateConfig(state.config.with_view_as(ViewAs::Grid)));

    let view_as_table_button = button(icon_table())
        .style(style::get_nav_button_style(view_as == ViewAs::Table))
        .on_press_with(|| Message::UpdateConfig(state.config.with_view_as(ViewAs::Table)));

    container(
        row![
            components::my_tooltip::view(view_as_grid_button, "View as Grid"),
            components::my_tooltip::view(view_as_table_button, "View as Table"),
        ]
        .spacing(5)
        .padding(5),
    )
    .style(style::card)
    .into()
}
