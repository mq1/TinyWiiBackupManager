// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_button::my_button, my_group::my_group},
};
use iced::{
    Element, padding,
    widget::{column, row, text_input},
};
use lucide_icons::Icon;

pub fn wiiload(state: &AppState) -> Element<'_, Message> {
    my_group(
        "Wiiload",
        column![
            "Wiiload is a method of loading .dol and .elf files over the network.\nAlso, you can use Wiiload to install homebrew applications directly onto your SD card.\nIf the icon in the very bottom right of the Homebrew Channel is lit up, it should work for you.\nPressing the home button in the Homebrew Channel will reveal your Wii's IP.",
            my_button()
                .icon(Icon::FileUp)
                .label("Choose a zip/dol/elf")
                .on_press(Message::PickFileToSendViaWiiload),
        ]
        .spacing(10)
    )
    .icon(Icon::MonitorUp)
    .top_right_content(
        row![
            "Type your Wii IP here →",
            text_input("192.168.1.100", &state.config.wii_ip)
                .on_input(Message::SetWiiIp)
                .padding(padding::left(3))
                .width(150)
        ]
        .spacing(5),
    )
    .into()
}
