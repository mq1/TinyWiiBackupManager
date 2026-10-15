// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    message::Message,
    ui::components::{my_card::my_card, row_button::row_button},
};
use iced::{
    Element, Length, padding,
    widget::{row, rule, space, text, tooltip},
};
use lucide_iced::{
    bytes::{BOX, HARD_DRIVE_DOWNLOAD, INFO, POINTER, TRASH},
    themed_icon,
};

pub fn game_row(game: &Game, is_exporting: bool) -> Element<'_, Message> {
    row![
        tooltip(
            if game.is_wii() {
                themed_icon(POINTER, 14.)
            } else {
                themed_icon(BOX, 14.)
            },
            my_card(if game.is_wii() { "Wii" } else { "GameCube" }),
            tooltip::Position::Top
        ),
        text!("{} [{}]", game.title(), game.id()),
        space::horizontal(),
        text(game.size_str()),
        rule::vertical(1),
        row_button(
            TRASH,
            "Delete game",
            Some(|| Message::AskDeleteDir(game.path().to_path_buf()))
        ),
        row_button(
            HARD_DRIVE_DOWNLOAD,
            "Export game",
            (!is_exporting).then_some(|| Message::PickExportDest(game.clone()))
        ),
        row_button(
            INFO,
            "Game info",
            Some(|| Message::OpenGameInfo(game.clone()))
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
