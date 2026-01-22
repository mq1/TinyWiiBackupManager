// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{hbc::app::HbcApp, message::Message, state::State, ui::Screen};
use iced::{
    Alignment, Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_info, icon_trash};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |(_, app): (usize, &HbcApp)| {
            text(app.meta().name())
        }),
        table::column(text("Version").size(16), |(_, app): (usize, &HbcApp)| {
            text(app.meta().version())
        }),
        table::column(text("Size").size(16), |(_, app): (usize, &HbcApp)| {
            text(app.size().to_string())
        }),
        table::column(text("Actions").size(16), |(i, app): (usize, &HbcApp)| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press(Message::NavigateTo(Screen::HbcInfo(i))),
                text('•'),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(move || Message::AskDeleteDirConfirmation(
                        app.path().to_path_buf(),
                    )),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    let table = if !state.hbc_filter.is_empty() {
        let iter = state.hbc_app_list.iter_enumerate_filtered();
        table(t_columns, iter).width(Length::Fill)
    } else {
        let iter = state.hbc_app_list.iter().enumerate();
        table(t_columns, iter).width(Length::Fill)
    };

    container(table).padding(10).into()
}
