// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{hbc::osc::OscAppMeta, message::Message, state::State};
use iced::{
    Alignment, Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_cloud_download, icon_info};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |(_, app): (usize, &OscAppMeta)| {
            text(&app.name)
        }),
        table::column(
            text("Version").size(16),
            |(_, app): (usize, &OscAppMeta)| text(&app.version),
        ),
        table::column(text("Author").size(16), |(_, app): (usize, &OscAppMeta)| {
            text(&app.author)
        }),
        table::column(text("Actions").size(16), |(i, _): (usize, &OscAppMeta)| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::NavToOscAppInfo(i)),
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
        let iter = state.osc_app_list.iter_enumerate_filtered();
        table(t_columns, iter).width(Length::Fill)
    } else {
        let iter = state.osc_app_list.iter().enumerate();
        table(t_columns, iter).width(Length::Fill)
    };

    container(table).padding(10).into()
}
