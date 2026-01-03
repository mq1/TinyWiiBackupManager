// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Element,
    widget::{button, column, container, space},
};
use lucide_icons::iced::{
    icon_gamepad_2, icon_hard_drive, icon_info, icon_shopping_bag, icon_waves,
};

pub fn view(state: &State) -> Element<'_, Message> {
    container(
        column![
            button(icon_gamepad_2().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::Games))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Games)),
            button(icon_waves().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::HbcApps))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::HbcApps)),
            button(icon_shopping_bag().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::Osc))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Osc)),
            space::vertical(),
            my_tooltip::view(
                button(icon_hard_drive().size(20).center())
                    .style(|t, s| style::nav_button(t, s, false))
                    .height(40)
                    .width(40)
                    .on_press(Message::SelectMountPoint),
                "Select Drive/Mount Point"
            ),
            button(icon_info().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::About))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::About)),
        ]
        .padding(10)
        .spacing(10),
    )
    .style(style::nav_container)
    .into()
}
