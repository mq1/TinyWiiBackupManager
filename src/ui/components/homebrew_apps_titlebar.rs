// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
        refresh_button::refresh_button,
        view_as::view_as,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space, text, tooltip},
};
use lucide_icons::{Icon, iced::icon_chevron_right};

pub fn homebrew_apps_titlebar(state: &AppState) -> Element<'_, Message> {
    row![
        icon_chevron_right().size(20),
        text("Homebrew Apps").size(20),
        space::horizontal(),
        view_as(state),
        space().width(5),
        refresh_button(state),
        tooltip(
            my_button(None, Icon::Plus, MyButtonKind::Toolbar).on_press(Message::PickHomebrewApps),
            my_card("Import app(s)"),
            tooltip::Position::Bottom
        )
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}
