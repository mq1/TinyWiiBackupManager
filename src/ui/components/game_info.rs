// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element,
    widget::{button, column, image, row, rule, space, stack, text},
};
use iced_fonts::lucide;

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    let col = column![
        row![lucide::gamepad_two().size(19), text(&game.title).size(18)].spacing(5),
        row![lucide::folder(), text("Path:"), text(game.get_path_str())].spacing(5),
        rule::horizontal(1),
        row![
            lucide::chevron_right().size(19),
            text("Disc Header").size(18)
        ]
        .spacing(5),
        row![lucide::tag(), text("ID:")].spacing(5),
        rule::horizontal(1),
        row![lucide::chevron_right().size(19), text("Disc Meta").size(18)].spacing(5),
        rule::horizontal(1),
        row![
            lucide::chevron_right().size(19),
            text("NKit Hashes").size(18)
        ]
        .spacing(5),
        rule::horizontal(1),
        row![
            lucide::chevron_right().size(19),
            text("GameTDB info").size(18)
        ]
        .spacing(5),
        space::vertical(),
        row![
            button(row![lucide::folder(), text("Open Game Directory")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenGameDir(game_i)),
            button(row![lucide::globe(), text("Open GameTDB Page")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenGameTdb(game_i)),
            button(row![lucide::trash(), text("Delete Game")].spacing(5))
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
            row![space::horizontal(), image(cover).height(248)].padding(10)
        ]
        .into(),
        None => col.into(),
    }
}
