// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Element,
    widget::{text, tooltip},
};
use lucide_icons::Icon;
use twbm_osc::manager::OscAppManager;

pub fn refresh_osc_button(state: &AppState) -> Element<'_, Message> {
    let mut refresh_btn = my_button().icon(Icon::RotateCw).kind(MyButtonKind::Toolbar);

    if !matches!(state.osc, OscAppManager::Loading) {
        refresh_btn = refresh_btn.on_press(Message::RefreshOscContents);
    }

    let tooltip_text = if let OscAppManager::Loaded(apps) = &state.osc {
        text!(
            "Refresh OSC contents cache (refreshed {:#} ago)",
            apps.last_refresh()
        )
    } else {
        text("Refresh OSC contents cache")
    };

    tooltip(
        refresh_btn,
        my_card(tooltip_text),
        tooltip::Position::Bottom,
    )
    .into()
}
