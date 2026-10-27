// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    ui::{components::my_button::my_button, pages::Page},
};
use iced::{
    Alignment, Element, Length,
    widget::{column, container},
};
use lucide_icons::Icon;

pub fn no_drive() -> Element<'static, Message> {
    container(
        column![
            "No drive selected. Add your Wii's USB disk or SD card in My Consoles.",
            my_button()
                .icon(Icon::MonitorSmartphone)
                .label("Open My Consoles")
                .on_press(Message::NavigateTo(Page::Consoles)),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .center(Length::Fill)
    .into()
}
