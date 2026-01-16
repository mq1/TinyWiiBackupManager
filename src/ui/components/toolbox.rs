// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, row, rule, space, text},
};
use lucide_icons::iced::{
    icon_apple, icon_cloud_download, icon_play, icon_tool_case, icon_wand_sparkles,
};

pub fn view(_state: &State) -> Element<'_, Message> {
    column![
        row![icon_tool_case().size(18), text("Toolbox").size(18)].spacing(5),
        container(
            column![
                row![icon_wand_sparkles(), text("USB Loader GX")].spacing(5),
                rule::horizontal(1),
                space(),
                row![
                    button(icon_cloud_download())
                        .style(style::rounded_button)
                        .on_press(Message::DownloadWiitdbToDrive),
                    text("Update wiitdb.xml (overwrites existing one)")
                ]
                .align_y(Alignment::Center)
                .spacing(10)
            ]
            .spacing(5)
            .padding(10)
            .width(Length::Fill)
        )
        .style(style::card),
        container(
            column![
                row![icon_apple(), text("macOS")].spacing(5),
                rule::horizontal(1),
                space(),
                row![
                    button(icon_play())
                        .style(style::rounded_button)
                        .on_press(Message::RunDotClean),
                    text("Delete ._ files (dot_clean)")
                ]
                .align_y(Alignment::Center)
                .spacing(10)
            ]
            .spacing(5)
            .padding(10)
            .width(Length::Fill)
        )
        .style(style::card)
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
