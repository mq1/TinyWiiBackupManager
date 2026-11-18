// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::components::my_card::my_card};
use iced::{
    Element,
    widget::{button, tooltip},
};
use lucide_icons::Icon;

pub fn row_button<'a>(
    icon: Icon,
    hint: &'static str,
    on_press: Option<impl Fn() -> Message + 'a>,
) -> Element<'a, Message> {
    let mut btn = button(icon.widget().center())
        .padding(0)
        .style(button::text)
        .width(20)
        .height(20);

    if let Some(on_press) = on_press {
        btn = btn.on_press_with(on_press);
    }

    tooltip(btn, my_card(hint), tooltip::Position::Top).into()
}
