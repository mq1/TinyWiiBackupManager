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

pub fn refresh_osc_button(state: &AppState) -> Element<'_, Message> {
    let mut refresh_btn = my_button().icon(Icon::RotateCw).kind(MyButtonKind::Toolbar);

    if !state.ongoing.contains(Ongoing::GettingOscContents) {
        refresh_btn = refresh_btn.on_press(Message::RefreshOscContents);
    }

    tooltip(
        refresh_btn,
        my_card("Refresh OSC contents cache"),
        tooltip::Position::Bottom,
    )
    .into()
}
