// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{games::game::Game, messages::Message};
use iced::{
    Element, padding,
    widget::{button, row, space, text},
};
use lucide_icons::iced::{icon_box, icon_info, icon_pointer};

pub fn view((idx, game): (usize, &Game)) -> Element<'_, Message> {
    let console_icon = if game.is_wii {
        icon_pointer()
    } else {
        icon_box()
    };

    row![
        console_icon,
        text!("{} [{}]", &game.title, game.id),
        space::horizontal(),
        button(icon_info().center())
            .padding(0)
            .on_press(Message::OpenGameInfo(idx))
            .style(button::text)
            .width(20)
            .height(20)
    ]
    .spacing(5)
    .padding(padding::all(2).left(5))
    .into()
}
