// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{drive_info::drive_info, my_button::my_button, my_group::my_group},
};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Column, column, row, scrollable, text},
};
use lucide_icons::{Icon, iced::icon_chevron_right};

pub fn toolbox(state: &AppState) -> Element<'_, Message> {
    let titlebar = row![icon_chevron_right().size(20), text("Toolbox").size(20)]
        .spacing(5)
        .padding(padding::all(14).left(10));

    let tool_groups = crate::toolbox::all()
        .map(|tool_group| {
            let tools = tool_group
                .items
                .iter()
                .map(|tool| {
                    row![
                        my_button()
                            .icon(Icon::Play)
                            .on_press(Message::RunTool(tool)),
                        text(tool.label)
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect::<Column<'_, _>>()
                .spacing(5);

            my_group(tool_group.label)
                .icon(tool_group.icon)
                .content(tools)
                .into()
        })
        .collect::<Column<'_, _>>()
        .spacing(10);

    let contents = scrollable(
        column![drive_info(state), tool_groups]
            .spacing(10)
            .padding(padding::left(10).bottom(10).right(20))
            .width(Length::Fill),
    );

    column![titlebar, contents].into()
}
