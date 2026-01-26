// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{hbc::osc::OscAppMeta, message::Message, state::State, ui::Screen};
use iced::{
    Alignment, Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_cloud_download, icon_info, icon_monitor_up};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |app: &OscAppMeta| text(app.name())),
        table::column(text("Version").size(16), |app: &OscAppMeta| {
            text(app.version())
        }),
        table::column(text("Author").size(16), |app: &OscAppMeta| {
            text(app.author())
        }),
        table::column(text("Actions").size(16), |app: &OscAppMeta| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(|| Message::NavTo(Screen::OscInfo(app.clone()))),
                text('•'),
                button(row![icon_monitor_up(), text("Wiiload")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(|| Message::WiiloadOsc(app.clone())),
                text('•'),
                button(row![icon_cloud_download(), text("Install")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(|| Message::AskInstallOscApp(app.clone()))
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    let table = if state.osc_filter.is_empty() {
        let iter = state.osc_app_list.iter();
        table(t_columns, iter).width(Length::Fill)
    } else {
        let iter = state.osc_app_list.iter_filtered();
        table(t_columns, iter).width(Length::Fill)
    };

    container(table).padding(10).into()
}
