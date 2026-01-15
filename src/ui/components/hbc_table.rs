// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{Screen, style},
};
use iced::{
    Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_info, icon_trash};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |i: usize| {
            text(&state.hbc_apps[i].meta.name)
        }),
        table::column(text("Version").size(16), |i: usize| {
            text(&state.hbc_apps[i].meta.version)
        }),
        table::column(text("Size").size(16), |i: usize| {
            text(state.hbc_apps[i].size.to_string())
        }),
        table::column(text("Actions").size(16), |i| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::HbcInfo(i))),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::AskDeleteHbcApp(i))
            ]
            .spacing(5)
        }),
    ];

    let table = if !state.hbc_filter.is_empty() {
        let indices = state.filtered_hbc_indices.iter().copied();
        table(t_columns, indices).width(Length::Fill)
    } else {
        let indices = 0..state.hbc_apps.len();
        table(t_columns, indices).width(Length::Fill)
    };

    container(table).padding(10).into()
}
