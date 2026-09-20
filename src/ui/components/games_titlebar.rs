// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::games_state::GameList,
    messages::Message,
    state::AppState,
    ui::components::{
        filter_console::filter_console,
        game_search::game_search,
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
        refresh_button::refresh_button,
        sort_by::sort_by,
        view_as::view_as,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space, tooltip},
};
use lucide_icons::Icon;

pub fn games_titlebar<'a>(games: &'a GameList, state: &'a AppState) -> Element<'a, Message> {
    row![
        game_search(games),
        space::horizontal(),
        filter_console(games),
        sort_by(state),
        view_as(state),
        space().width(5),
        refresh_button(state),
        tooltip(
            my_button()
                .icon(Icon::Plus)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::PickGames),
            my_card("Import game(s)"),
            tooltip::Position::Bottom
        ),
        tooltip(
            my_button()
                .icon(Icon::FolderPlus)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::PickGamesRecursively),
            my_card("Import game(s) recursively"),
            tooltip::Position::Bottom
        )
    ]
    .spacing(5)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
