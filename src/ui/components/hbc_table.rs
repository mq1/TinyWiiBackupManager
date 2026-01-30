// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    hbc::app::HbcApp,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip},
};
use iced::{
    Alignment, Element, Length,
    widget::{button, container, row, table, text},
};
use lucide_icons::iced::{icon_circle_arrow_up, icon_info, icon_trash};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |app: &HbcApp| {
            let mut row = row![text(app.meta().name())].spacing(5);

            if let Some(osc_i) = app.osc_i() {
                let osc_app = state.osc_app_list.get_unchecked(osc_i);

                if app.meta().version() != osc_app.version() {
                    let btn = button(icon_circle_arrow_up())
                        .style(button::text)
                        .padding(0)
                        .on_press_with(|| Message::AskInstallOscApp(osc_app.clone()));
                    row = row.push(my_tooltip::view(btn, "Update to latest version"));
                }
            }

            row
        }),
        table::column(text("Version").size(16), |app: &HbcApp| {
            container(text(app.meta().version()).wrapping(text::Wrapping::WordOrGlyph))
                .max_width(100)
        }),
        table::column(text("Author").size(16), |app: &HbcApp| {
            container(text(app.meta().coder()).wrapping(text::Wrapping::WordOrGlyph)).max_width(100)
        }),
        table::column(text("Size").size(16), |app: &HbcApp| {
            text(app.size().to_string())
        }),
        table::column(text("Actions").size(16), |app: &HbcApp| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(|| Message::NavTo(Screen::HbcInfo(app.clone()))),
                text('•'),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .padding(0)
                    .style(button::text)
                    .on_press_with(|| Message::AskDeleteDirConfirmation(app.path().clone())),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
        }),
    ];

    let table = if state.hbc_filter.is_empty() {
        let iter = state.hbc_app_list.iter();
        table(t_columns, iter).width(Length::Fill)
    } else {
        let iter = state.hbc_app_list.iter_filtered();
        table(t_columns, iter).width(Length::Fill)
    };

    container(table).padding(10).into()
}
