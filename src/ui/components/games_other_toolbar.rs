// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::game_search::game_search};
use iced::{Element, widget::row};

pub fn games_other_toolbar(state: &AppState) -> Element<'_, Message> {
    row![game_search(state)].into()
}
