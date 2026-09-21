// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, osc::osc_app::OscApp, ui::components::my_card::my_card};
use iced::{
    Element, Length, padding,
    widget::{button, row, rule, space, text, tooltip},
};
use lucide_icons::iced::icon_info;

pub fn osc_app_row(app: &OscApp) -> Element<'_, Message> {
    row![
        text!("{} ({})", app.name(), app.version()),
        space::horizontal(),
        text(app.size_str()),
        rule::vertical(1),
        tooltip(
            button(icon_info().center())
                .padding(0)
                .style(button::text)
                .width(20)
                .height(20)
                .on_press(Message::OpenOscAppInfo(app.clone())),
            my_card("App info"),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
