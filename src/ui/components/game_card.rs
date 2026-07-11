// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message, ui::components};
use iced::{
    Element,
    widget::{column, image, text},
};

pub fn view<'a>(game: &'a Game) -> Element<'a, Message> {
    components::card::view(column![text(&game.title), image(&game.cached_cover_path)]).into()
}
