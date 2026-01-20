// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element, Length, padding,
    widget::{button, column, image, row, rule, scrollable, space, stack, text},
};
use itertools::Itertools;
use lucide_icons::iced::{
    icon_baby, icon_badge_check, icon_bow_arrow, icon_box, icon_building, icon_calendar,
    icon_chevron_right, icon_disc_3, icon_earth, icon_file_archive, icon_fingerprint_pattern,
    icon_folder, icon_gamepad_2, icon_globe, icon_hash, icon_joystick, icon_languages,
    icon_lock_open, icon_notebook_pen, icon_pin, icon_pointer, icon_ruler_dimension_line, icon_tag,
    icon_trash, icon_triangle_alert, icon_user, icon_weight, icon_wifi,
};

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    let disc_info = match &game.disc_info {
        None => column![text("Loading Disc Info...")],
        Some(Err(e)) => column![text!("Could not load disc info: {}", e)],
        Some(Ok(disc_info)) => {
            let block_size = match disc_info.block_size {
                None => text("Block Size: N/A"),
                Some(size) => text!("Block Size: {size}"),
            };

            let disc_size = match disc_info.disc_size {
                None => text("Disc Size: N/A"),
                Some(size) => text!("Disc Size: {size}"),
            };

            let crc32 = match disc_info.crc32 {
                None => text("CRC32: N/A"),
                Some(crc32) => text!("CRC32: {crc32:02x}"),
            };

            let md5 = match disc_info.md5 {
                None => text("MD5: N/A"),
                Some(md5) => text!("MD5: {}", hex::encode(md5)),
            };

            let sha1 = match disc_info.sha1 {
                None => text("SHA1: N/A"),
                Some(sha1) => text!("SHA1: {}", hex::encode(sha1)),
            };

            let xxh64 = match disc_info.xxh64 {
                None => text("XXH64: N/A"),
                Some(xxh64) => text!("XXH64: {xxh64:02x}"),
            };

            column![
                row![icon_chevron_right().size(19), text("Disc Header").size(18)].spacing(5),
                row![icon_tag(), text!("ID: {}", disc_info.id.as_str())].spacing(5),
                row![
                    icon_notebook_pen(),
                    text!("Embedded Title: {}", &disc_info.title),
                ]
                .spacing(5),
                row![
                    icon_globe(),
                    text!(
                        "Region (inferred from ID): {}",
                        disc_info.id.as_region_str()
                    ),
                ]
                .spacing(5),
                row![icon_pointer(), text!("Is Wii: {}", disc_info.is_wii)].spacing(5),
                row![icon_box(), text!("Is GameCube: {}", disc_info.is_gc)].spacing(5),
                row![icon_hash(), text!("Disc Number: {}", disc_info.disc_num)].spacing(5),
                row![
                    icon_pin(),
                    text!("Disc Version: {}", disc_info.disc_version),
                ]
                .spacing(5),
                rule::horizontal(1),
                row![icon_chevron_right().size(19), text("Disc Meta").size(18)].spacing(5),
                row![icon_disc_3(), text!("Format: {}", disc_info.format)].spacing(5),
                row![
                    icon_file_archive(),
                    text!("Compression: {}", disc_info.compression)
                ]
                .spacing(5),
                row![icon_ruler_dimension_line(), block_size].spacing(5),
                row![
                    icon_lock_open(),
                    text!("Decrypted: {}", disc_info.decrypted)
                ]
                .spacing(5),
                row![
                    icon_triangle_alert(),
                    text!("Needs Hash Recovery: {}", disc_info.needs_hash_recovery)
                ]
                .spacing(5),
                row![
                    icon_badge_check(),
                    text!("Lossless: {}", disc_info.lossless)
                ]
                .spacing(5),
                row![icon_weight(), disc_size].spacing(5),
                rule::horizontal(1),
                row![icon_chevron_right().size(19), text("NKit Hashes").size(18)].spacing(5),
                row![icon_fingerprint_pattern(), crc32].spacing(5),
                row![icon_fingerprint_pattern(), md5].spacing(5),
                row![icon_fingerprint_pattern(), sha1].spacing(5),
                row![icon_fingerprint_pattern(), xxh64].spacing(5),
            ]
            .spacing(5)
        }
    };

    let wiitdb_info = match &game.wiitdb_info {
        None => column![text("GameTDB game info not found")],
        Some(info) => column![
            row![icon_chevron_right().size(19), text("GameTDB Info").size(18)].spacing(5),
            row![icon_tag(), text!("Name: {}", &info.name)].spacing(5),
            row![icon_earth(), text!("Region: {}", info.region)].spacing(5),
            row![
                icon_languages(),
                text!(
                    "Languages: {}",
                    info.languages.iter().map(<&'static str>::from).join(", ")
                )
            ]
            .spacing(5),
            row![
                icon_user(),
                text!(
                    "Developer: {}",
                    info.developer.as_deref().unwrap_or("Unknown")
                )
            ]
            .spacing(5),
            row![
                icon_building(),
                text!(
                    "Publisher: {}",
                    info.publisher.as_deref().unwrap_or("Unknown")
                )
            ]
            .spacing(5),
            row![
                icon_calendar(),
                text!(
                    "Date: {}-{}-{}",
                    info.date.year,
                    info.date.month,
                    info.date.day
                )
            ]
            .spacing(5),
            row![icon_bow_arrow(), text!("Genres: {}", info.genre.join(", "))].spacing(5),
            row![
                icon_baby(),
                text!("Rating: {} {}", &info.rating.r#type, &info.rating.value)
            ]
            .spacing(5),
            row![
                icon_wifi(),
                text!(
                    "WiFi: {} Players • {}",
                    info.wifi.players,
                    info.wifi
                        .features
                        .iter()
                        .map(<&'static str>::from)
                        .join(", ")
                )
            ]
            .spacing(5),
            row![
                icon_joystick(),
                text!(
                    "Input: {} Players • {}",
                    info.input.players,
                    info.input
                        .controls
                        .iter()
                        .map(|c| format!(
                            "{} ({})",
                            c.r#type,
                            if c.required { "Required" } else { "Optional" }
                        ))
                        .join(", ")
                )
            ]
            .spacing(5),
        ]
        .spacing(5),
    };

    let col = column![
        row![icon_gamepad_2().size(19), text(&game.title).size(18)].spacing(5),
        row![icon_folder(), text!("Path: {}", game.get_path_str())].spacing(5),
        rule::horizontal(1),
        scrollable(
            column![disc_info, rule::horizontal(1), wiitdb_info]
                .spacing(5)
                .width(Length::Fill)
        )
        .height(Length::Fill),
        rule::horizontal(1),
        row![
            button(row![icon_folder(), text("Open Game Directory")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(game.get_path_uri())),
            button(row![icon_globe(), text("Open GameTDB Page")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(game.get_gametdb_uri())),
            button(row![icon_trash(), text("Delete Game")].spacing(5))
                .style(style::rounded_danger_button)
                .on_press_with(|| Message::AskDeleteDirConfirmation(game.path.clone()))
        ]
        .spacing(5)
        .padding(padding::top(5))
    ]
    .spacing(5)
    .padding(10);

    match state.get_game_cover(game) {
        Some(cover) => stack![
            col,
            row![
                space::horizontal(),
                image(cover)
                    .height(186)
                    .filter_method(image::FilterMethod::Linear)
            ]
            .padding(10)
        ]
        .into(),
        None => col.into(),
    }
}
