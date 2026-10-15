// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ui::wii_ip::wii_ip,
    message::Message,
    state::AppState,
    ui::components::{my_button::my_button, my_group::my_group},
};
use iced::{
    Element,
    widget::{column, row, space},
};
use lucide_iced::bytes::{FILE_UP, MONITOR_UP};

pub fn wiiload(state: &AppState) -> Element<'_, Message> {
    my_group(
        "Wiiload",
        column![
            "Wiiload is a method of loading .dol and .elf files over the network.\nAlso, you can use Wiiload to install homebrew applications directly onto your SD card.\nIf the icon in the very bottom right of the Homebrew Channel is lit up, it should work for you.\nPressing the home button in the Homebrew Channel will reveal your Wii's IP.",
            row![
                my_button()
                    .icon(FILE_UP)
                    .label("Choose a zip/dol/elf")
                    .on_press(Message::PickFileToSendViaWiiload),
                space::horizontal(),
                wii_ip(&state.config).map(Message::Config),
            ]
        ]
        .spacing(10)
    )
    .icon(MONITOR_UP)
    .into()
}
