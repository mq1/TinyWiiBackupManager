// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element,
    widget::{button, column, image, row, space, text},
};
use lucide_icons::iced::{icon_folder, icon_gamepad_2, icon_trash};

pub fn view(state: &State, game_i: usize) -> Element<'_, Message> {
    let game = &state.games[game_i];

    let mut row = row![
        column![
            row![icon_gamepad_2().size(19), text(&game.title).size(18)].spacing(5),
            row![icon_folder(), text("Path:"), text(game.get_path_str())].spacing(5),
            space::vertical(),
            row![
                button(row![icon_folder(), text("Open Game Directory")].spacing(5))
                    .style(style::rounded_button)
                    .on_press(Message::OpenGameDir(game_i)),
                button(row![icon_trash(), text("Delete Game")].spacing(5))
                    .style(style::rounded_danger_button)
                    .on_press(Message::AskDeleteGame(game_i))
            ]
            .spacing(5)
        ]
        .spacing(5),
        space::horizontal()
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
