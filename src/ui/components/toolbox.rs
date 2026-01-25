// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, Length,
    widget::{button, column, container, row, rule, space, text},
};
use lucide_icons::iced::{
    icon_cloud_download, icon_image_down, icon_skull, icon_tool_case, icon_wand_sparkles,
};

#[cfg(target_os = "macos")]
use lucide_icons::iced::{icon_apple, icon_play};

pub fn view(_state: &State) -> Element<'_, Message> {
    let usbloader_gx = column![
        row![icon_wand_sparkles(), "USB Loader GX"].spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_cloud_download())
                .style(style::rounded_button)
                .on_press(Message::DownloadWiitdbToDrive),
            "Update wiitdb.xml (overwrites existing one)"
        ]
        .align_y(Alignment::Center)
        .spacing(10),
        row![
            button(icon_image_down())
                .style(style::rounded_button)
                .on_press(Message::DownloadCoversForUsbLoaderGx),
            "Download all covers (defaults to English for PAL games; usbloader_gx downloads them in the correct language)"
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    let wiiflow = column![
        row![icon_wand_sparkles(), "WiiFlow Lite"].spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_image_down())
                .style(style::rounded_button)
                .on_press(Message::DownloadCoversForWiiflow),
            "Download all covers (defaults to English for PAL games)"
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    let cheats = column![
        row![icon_skull(), "Cheats (txtcodes)"].spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_cloud_download())
                .style(style::rounded_button)
                .on_press(Message::DownloadCheatsForAllGames),
            "Download cheats for all games"
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    #[cfg(not(target_os = "macos"))]
    let os_specific = space();

    #[cfg(target_os = "macos")]
    let os_specific = container(
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
        .width(Length::Fill),
    )
    .style(style::card);

    column![
        row![icon_tool_case().size(18), text("Toolbox").size(18)].spacing(5),
        container(usbloader_gx).style(style::card),
        container(wiiflow).style(style::card),
        container(cheats).style(style::card),
        os_specific
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
