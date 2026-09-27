// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{
        convert::convert, drive_info::drive_info, my_button::my_button, my_group::my_group,
        wiiload::wiiload,
    },
};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Column, column, row, scrollable, text},
};
use lucide_icons::Icon;

pub fn toolbox(state: &AppState) -> Element<'_, Message> {
    let has_drive = !state.config.mount_point.as_os_str().is_empty();

    let tool_groups = crate::toolbox::all()
        .map(|tool_group| {
            let tools = tool_group
                .items()
                .map(|tool| {
                    row![
                        my_button()
                            .icon(Icon::Play)
                            .on_press(Message::RunTool(tool)),
                        text(tool.label())
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .into()
                })
                .collect::<Column<'_, _>>()
                .spacing(5);

            my_group(tool_group.label(), tools)
                .icon(tool_group.icon())
                .into()
        })
        .collect::<Column<'_, _>>()
        .spacing(10);

    scrollable(
        column![
            has_drive.then_some(drive_info(state)),
            wiiload(state),
            convert(),
            has_drive.then_some(tool_groups)
        ]
        .spacing(10)
        .padding(padding::all(10).right(20))
        .width(Length::Fill),
    )
    .into()
}
