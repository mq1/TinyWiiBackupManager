// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::HbcApp,
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
        table::column(text("Name").size(16), |(_, app): (usize, &HbcApp)| {
            text(&app.meta.name)
        }),
        table::column(text("Version").size(16), |(_, app): (usize, &HbcApp)| {
            text(&app.meta.version)
        }),
        table::column(text("Size").size(16), |(_, app): (usize, &HbcApp)| {
            text(app.size.to_string())
        }),
        table::column(text("Actions").size(16), |(i, _): (usize, &HbcApp)| {
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
        let apps = state
            .filtered_hbc_indices
            .iter()
            .copied()
            .map(|i| (i, &state.hbc_apps[i]));
        table(t_columns, apps).width(Length::Fill)
    } else {
        let apps = state.hbc_apps.iter().enumerate();
        table(t_columns, apps).width(Length::Fill)
    };

    container(table).padding(10).into()
}
