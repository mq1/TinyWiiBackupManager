// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, Length, padding,
    widget::{button, column, container, row, rule, scrollable, space, text, text_input},
};
use lucide_icons::iced::{
    icon_brush_cleaning, icon_cloud_download, icon_file_up, icon_hard_drive_download,
    icon_image_down, icon_monitor_up, icon_play, icon_skull, icon_tool_case, icon_wand_sparkles,
    icon_wifi_pen,
};

#[cfg(target_os = "macos")]
use lucide_icons::iced::icon_apple;

#[allow(clippy::too_many_lines)]
pub fn view(state: &State) -> Element<'_, Message> {
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
            "Download all covers\ndefaults to English for PAL games (usbloader_gx downloads them in the correct language)"
        ]
        .align_y(Alignment::Center)
        .spacing(10),
        row![
            button(icon_image_down())
                .style(style::rounded_button)
                .on_press(Message::DownloadBanners),
            "Download banners for all GameCube games"
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

    let cleanup = column![
        row![icon_brush_cleaning(), "Cleanup"].spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_play())
                .style(style::rounded_button)
                .on_press(Message::NormalizePaths),
            "Normalize paths (makes sure the game directories' layouts are correct)"
        ]
        .align_y(Alignment::Center)
        .spacing(10),
        row![
            button(icon_play())
                .style(style::rounded_button)
                .on_press(Message::ConfirmStripAllGames),
            "Remove the update partition from all .wbfs files"
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    let wiiload = column![
        row![icon_monitor_up(), "Wiiload"].spacing(5),
        rule::horizontal(1),
        text("Wiiload is a method of loading .dol and .elf files over the network."),
        text("Also, you can use Wiiload to install homebrew applications directly onto your SD card."),
        text("If the icon in the very bottom right of the Homebrew Channel is lit up, it should work for you."),
        text("Pressing the home button in the Homebrew Channel will reveal your Wii's IP."),
        rule::horizontal(1),
        space(),
        row![
            button(icon_file_up())
                .style(style::rounded_button).on_press(Message::ChooseFileToWiiload),
            "Choose a file to send (.zip/.dol/.elf)",
            space::horizontal(),
            icon_wifi_pen(),
            "Wii IP:",
            text_input("192.168.1.100", state.config.wii_ip())
                .width(130)
                .style(style::rounded_text_input)
                .padding(padding::left(10).vertical(5))
                .on_input(|wii_ip| Message::UpdateConfig(
                    state.config.clone_with_wii_ip(wii_ip)
                ))
        ]
        .align_y(Alignment::Center)
        .spacing(10),
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    let manual_archive_button: Element<'_, Message> = if state.manual_archiving_game.is_some() {
        button(row![icon_play(), "Archive"].spacing(5))
            .style(style::rounded_button)
            .on_press(Message::RunManualGameArchiving)
            .into()
    } else {
        space().into()
    };

    let manual_archive = column![
        row![
            icon_hard_drive_download(),
            "Choose game to archive manually (a plain conversion leveraging",
            button(text("NOD").style(text::primary))
                .style(button::text)
                .padding(0)
                .on_press_with(|| Message::OpenThat("https://github.com/encounter/nod".into())),
            ")"
        ]
        .spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_file_up())
                .style(style::rounded_button)
                .on_press(Message::ChooseGameToArchiveManually),
            text(
                state
                    .manual_archiving_game
                    .as_ref()
                    .map_or("No game selected".to_string(), |p| p
                        .to_string_lossy()
                        .to_string())
            ),
            space::horizontal(),
            manual_archive_button
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    #[cfg(target_os = "macos")]
    let macos = column![
        row![icon_apple(), text("macOS")].spacing(5),
        rule::horizontal(1),
        space(),
        row![
            button(icon_brush_cleaning())
                .style(style::rounded_button)
                .on_press(Message::RunDotClean),
            text("Delete ._ files (dot_clean)")
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fill);

    let mut col = column![
        container(usbloader_gx).style(style::card),
        container(wiiflow).style(style::card),
        container(cheats).style(style::card),
        container(cleanup).style(style::card),
        container(wiiload).style(style::card),
        container(manual_archive).style(style::card),
    ]
    .spacing(10)
    .padding(padding::horizontal(10));

    #[cfg(target_os = "macos")]
    {
        col = col.push(container(macos).style(style::card));
    }

    col = col.push(space());

    column![
        row![icon_tool_case().size(18), text("Toolbox").size(18)]
            .spacing(5)
            .padding(padding::top(10).left(10)),
        scrollable(col).spacing(1).height(Length::Fill)
    ]
    .spacing(10)
    .into()
}
