// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ui::{sort_by::sort_by, view_as::view_as},
    homebrew::homebrew_state::HomebrewAppList,
    message::Message,
    state::AppState,
    ui::components::{
        homebrew_app_search::homebrew_app_search,
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
        refresh_button::refresh_button,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space, tooltip},
};
use lucide_icons::Icon;

pub fn homebrew_apps_titlebar<'a>(
    apps: &'a HomebrewAppList,
    state: &'a AppState,
) -> Element<'a, Message> {
    row![
        homebrew_app_search(apps),
        space::horizontal(),
        sort_by(&state.config).map(Message::Config),
        view_as(&state.config).map(Message::Config),
        space().width(5),
        refresh_button(state),
        tooltip(
            my_button()
                .icon(Icon::Plus)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::PickHomebrewApps),
            my_card("Import app(s)"),
            tooltip::Position::Bottom
        )
    ]
    .spacing(5)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
