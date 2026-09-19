// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::games_state::GamesState,
    homebrew::homebrew_state::HomebrewState,
    messages::Message,
    osc::osc_state::OscState,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{Element, widget::tooltip};
use lucide_icons::Icon;

pub fn refresh_button(state: &AppState) -> Element<'_, Message> {
    let mut refresh_btn = my_button().icon(Icon::RotateCw).kind(MyButtonKind::Toolbar);

    if !matches!(state.games, GamesState::Loading)
        && !matches!(state.homebrew, HomebrewState::Loading)
        && !matches!(state.osc_contents, OscState::Loading)
    {
        refresh_btn = refresh_btn.on_press(Message::RefreshGamesAndApps);
    }

    tooltip(
        refresh_btn,
        my_card("Refresh games and apps"),
        tooltip::Position::Bottom,
    )
    .into()
}
