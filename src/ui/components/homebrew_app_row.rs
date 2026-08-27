// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp, messages::Message, ui::components::my_card::my_card,
};
use iced::{
    Element, Length, padding,
    widget::{button, row, rule, space, text, tooltip},
};
use lucide_icons::iced::{icon_info, icon_trash};
use std::sync::Arc;

pub fn homebrew_app_row(app: &Arc<HomebrewApp>) -> Element<'_, Message> {
    row![
        text!("{} ({})", &app.meta.name, app.meta.version),
        space::horizontal(),
        text(app.size.to_string()),
        rule::vertical(1),
        tooltip(
            button(icon_trash().center())
                .padding(0)
                .on_press_with(|| Message::AskDeleteDir(app.path.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("Delete app"),
            tooltip::Position::Top
        ),
        tooltip(
            button(icon_info().center())
                .padding(0)
                .on_press_with(|| Message::OpenHomebrewAppInfo(app.clone()))
                .style(button::text)
                .width(20)
                .height(20),
            my_card("App info"),
            tooltip::Position::Top
        ),
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(2).left(5))
    .into()
}
