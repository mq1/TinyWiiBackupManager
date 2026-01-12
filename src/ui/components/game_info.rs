// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{game_id::GameID, message::Message, state::State, ui::style};
use iced::{
    Element,
    widget::{button, column, image, row, rule, space, stack, text},
};
use lucide_icons::iced::{
    icon_box, icon_chevron_right, icon_folder, icon_gamepad_2, icon_globe, icon_hash,
    icon_notebook_pen, icon_pin, icon_pointer, icon_tag, icon_trash,
};

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    let disc_info = match &game.disc_info {
        None => column![text("Loading Disc Info...")],
        Some(Err(e)) => column![text(e)],
        Some(Ok(disc_info)) => column![
            row![icon_chevron_right().size(19), text("Disc Header").size(18)].spacing(5),
            row![
                icon_tag(),
                text("ID:"),
                text(disc_info.id.as_str().to_string())
            ]
            .spacing(5),
            row![
                icon_notebook_pen(),
                text("Embedded Title:"),
                text(disc_info.title.to_string())
            ]
            .spacing(5),
            row![
                icon_globe(),
                text("Region (inferred from ID):"),
                text(disc_info.id.as_region_str().to_string())
            ]
            .spacing(5),
            row![
                icon_pointer(),
                text("Is Wii:"),
                text(disc_info.is_wii.to_string())
            ]
            .spacing(5),
            row![
                icon_box(),
                text("Is GameCube:"),
                text(disc_info.is_gc.to_string())
            ]
            .spacing(5),
            row![
                icon_hash(),
                text("Disc Number:"),
                text(disc_info.disc_num.to_string())
            ]
            .spacing(5),
            row![
                icon_pin(),
                text("Disc Version:"),
                text(disc_info.disc_version.to_string())
            ]
            .spacing(5),
        ]
        .spacing(5),
    };

    let col = column![
        row![icon_gamepad_2().size(19), text(&game.title).size(18)].spacing(5),
        row![icon_folder(), text("Path:"), text(game.get_path_str())].spacing(5),
        rule::horizontal(1),
        disc_info,
        rule::horizontal(1),
        row![icon_chevron_right().size(19), text("Disc Meta").size(18)].spacing(5),
        rule::horizontal(1),
        row![icon_chevron_right().size(19), text("NKit Hashes").size(18)].spacing(5),
        rule::horizontal(1),
        row![icon_chevron_right().size(19), text("GameTDB info").size(18)].spacing(5),
        space::vertical(),
        row![
            button(row![icon_folder(), text("Open Game Directory")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenGameDir(game_i)),
            button(row![icon_globe(), text("Open GameTDB Page")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenGameTdb(game_i)),
            button(row![icon_trash(), text("Delete Game")].spacing(5))
                .style(style::rounded_danger_button)
                .on_press(Message::AskDeleteGame(game_i))
        ]
        .spacing(5)
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
