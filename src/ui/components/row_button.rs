// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::components::my_card::my_card};
use iced::{
    Element,
    widget::{button, tooltip},
};
use lucide_iced::themed_icon;

pub fn row_button<'a>(
    icon_bytes: &'static [u8],
    hint: &'static str,
    on_press: Option<impl Fn() -> Message + 'a>,
) -> Element<'a, Message> {
    let mut btn = button(themed_icon(icon_bytes, 14.))
        .padding(0)
        .style(button::text)
        .width(20)
        .height(20);

    if let Some(on_press) = on_press {
        btn = btn.on_press_with(on_press);
    }

    tooltip(btn, my_card(hint), tooltip::Position::Top).into()
}
