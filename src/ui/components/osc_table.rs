// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::Screen};
use iced::{
    Alignment, Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_cloud_download, icon_info};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |i: usize| {
            text(&state.osc_apps[i].name)
        }),
        table::column(text("Version").size(16), |i: usize| {
            text(&state.osc_apps[i].version)
        }),
        table::column(text("Author").size(16), |i: usize| {
            text(&state.osc_apps[i].author)
        }),
        table::column(text("Actions").size(16), |i| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::NavigateTo(Screen::OscInfo(i))),
                text('•'),
                button(row![icon_cloud_download(), text("Install")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::AskInstallOscApp(i))
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    let table = if !state.osc_filter.is_empty() {
        let indices = state.filtered_osc_indices.iter().copied();
        table(t_columns, indices).width(Length::Fill)
    } else {
        let indices = 0..state.osc_apps.len();
        table(t_columns, indices).width(Length::Fill)
    };

    container(table).padding(10).into()
}
