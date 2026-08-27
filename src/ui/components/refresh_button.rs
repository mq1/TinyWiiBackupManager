// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::{AppState, Ongoing},
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{Element, widget::tooltip};
use lucide_icons::Icon;

pub fn refresh_button(state: &AppState) -> Element<'_, Message> {
    let mut refresh_btn = my_button(None, Icon::RotateCw, MyButtonKind::Toolbar);

    if !state.ongoing.intersects(
        Ongoing::GettingGames | Ongoing::GettingHomebrewApps | Ongoing::GettingDriveInfo,
    ) {
        refresh_btn = refresh_btn.on_press(Message::RefreshGamesAndApps);
    }

    tooltip(
        refresh_btn,
        my_card("Refresh games and apps"),
        tooltip::Position::Bottom,
    )
    .into()
}
