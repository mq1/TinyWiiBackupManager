// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Element,
    widget::{button, tooltip},
};
use lucide_icons::Icon;
use tap::Pipe;

pub fn row_button<'a>(
    icon: Icon,
    hint: &'static str,
    on_press: Option<impl Fn() -> Message + 'a>,
) -> Element<'a, Message> {
    let btn = button(icon.widget().center())
        .padding(0)
        .style(button::text)
        .width(20)
        .height(20)
        .pipe(|btn| {
            if let Some(on_press) = on_press {
                btn.on_press_with(on_press)
            } else {
                btn
            }
        });

    tooltip(btn, my_card(hint), tooltip::Position::Top).into()
}
