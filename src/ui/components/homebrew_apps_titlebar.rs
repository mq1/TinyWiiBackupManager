// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space, text, tooltip},
};
use lucide_icons::{Icon, iced::icon_chevron_right};

pub fn view<'a>() -> Element<'a, Message> {
    row![
        icon_chevron_right().size(20),
        text("Homebrew Apps").size(20),
        space::horizontal(),
        tooltip(
            my_button(None, Icon::RotateCw, MyButtonKind::Toolbar)
                .on_press(Message::RefreshGamesAndApps),
            my_card("Refresh games and apps"),
            tooltip::Position::Bottom
        )
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .into()
}
