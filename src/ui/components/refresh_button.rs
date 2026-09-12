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
use tap::Pipe;

pub fn refresh_button(state: &AppState) -> Element<'_, Message> {
    let refresh_btn = my_button()
        .icon(Icon::RotateCw)
        .kind(MyButtonKind::Toolbar)
        .pipe(|btn| {
            if state.ongoing.is_disjoint(
                Ongoing::GettingGames | Ongoing::GettingHomebrewApps | Ongoing::GettingDriveInfo,
            ) {
                btn.on_press(Message::RefreshGamesAndApps)
            } else {
                btn
            }
        });

    tooltip(
        refresh_btn,
        my_card("Refresh games and apps"),
        tooltip::Position::Bottom,
    )
    .into()
}
