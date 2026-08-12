// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{self, my_button::MyButton, view_as},
};
use iced::{
    Alignment, Element,
    widget::{row, space, text, tooltip},
};
use lucide_icons::{Icon, iced::icon_chevron_right};

pub fn view(state: &AppState) -> Element<'_, Message> {
    row![
        icon_chevron_right().size(20),
        text("Games").size(20),
        space::horizontal(),
        view_as::view(state),
        tooltip(
            MyButton::new()
                .icon(Icon::RotateCw)
                .toolbar()
                .view()
                .on_press(Message::RefreshGamesAndApps),
            components::card::view("Refresh games and apps"),
            tooltip::Position::Bottom
        )
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}
