// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::games_state::GameList, messages::Message, ui::components::game_search::game_search,
    util::drive_state::DriveState,
};
use iced::{
    Alignment, Element,
    widget::{Text, row, space, text},
};

fn usage<'a>(games: &'a GameList, drive: &'a DriveState) -> Text<'a> {
    text!(
        "{} game{}   ~ {}  ",
        games.count(),
        if games.count() == 1 { "" } else { "s" },
        if let DriveState::Loaded(info) = &drive {
            info.games_size_str()
        } else {
            ""
        }
    )
}

pub fn games_other_toolbar<'a>(games: &'a GameList, drive: &'a DriveState) -> Element<'a, Message> {
    row![game_search(games), space::horizontal(), usage(games, drive)]
        .align_y(Alignment::Center)
        .into()
}
