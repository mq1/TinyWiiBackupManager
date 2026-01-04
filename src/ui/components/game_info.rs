// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element,
    widget::{button, column, image, row, rule, space, text},
};
use lucide_icons::iced::{
    icon_chevron_right, icon_folder, icon_gamepad_2, icon_globe, icon_tag, icon_trash,
};

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    let mut row = row![
        column![
            row![icon_gamepad_2().size(19), text(&game.title).size(18)].spacing(5),
            row![icon_folder(), text("Path:"), text(game.get_path_str())].spacing(5),
            rule::horizontal(1),
            row![icon_chevron_right().size(19), text("Disc Header").size(18)].spacing(5),
            row![icon_tag(), text("ID:")].spacing(5),
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
        .spacing(5),
    ]
    .padding(10);

    if let Some(cover) = state.get_game_cover(game) {
        row = row.push(
            button(image(cover).height(100))
                .style(button::text)
                .on_press(Message::OpenGameCover(game_i)),
        );
    }

    row.into()
}
