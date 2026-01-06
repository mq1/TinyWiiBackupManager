// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ThemePreference,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
};
use iced::{
    Element,
    widget::{button, column, container, space},
};
use iced_fonts::lucide;

pub fn view(state: &State) -> Element<'_, Message> {
    let theme_icon = match state.config.get_theme_pref() {
        ThemePreference::Light => lucide::sun(),
        ThemePreference::Dark => lucide::moon(),
        ThemePreference::System => lucide::sun_moon(),
    };

    container(
        column![
            button(lucide::gamepad_two().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::Games))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Games)),
            button(lucide::waves().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::HbcApps))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::HbcApps)),
            button(lucide::shopping_bag().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::Osc))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Osc)),
            button(lucide::monitor_up().size(20).center())
                .style(|t, s| style::nav_button(t, s, false))
                .height(40)
                .width(40),
            button(lucide::git_compare_arrows().size(20).center())
                .style(|t, s| style::nav_button(t, s, false))
                .height(40)
                .width(40),
            button(lucide::wrench().size(20).center())
                .style(|t, s| style::nav_button(t, s, false))
                .height(40)
                .width(40),
            button(lucide::settings().size(20).center())
                .style(|t, s| style::nav_button(t, s, false))
                .height(40)
                .width(40),
            space::vertical(),
            my_tooltip::view(
                button(lucide::hard_drive().size(20).center())
                    .style(|t, s| style::nav_button(t, s, false))
                    .height(40)
                    .width(40)
                    .on_press(Message::SelectMountPoint),
                "Select Drive/Mount Point"
            ),
            my_tooltip::view(
                button(theme_icon.size(20).center())
                    .style(|t, s| style::nav_button(t, s, false))
                    .height(40)
                    .width(40)
                    .on_press(Message::ChangeTheme),
                "Change Theme (Light/Dark/System)"
            ),
            button(lucide::info().size(20).center())
                .style(|t, s| style::nav_button(t, s, state.screen == Screen::About))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::About)),
        ]
        .padding(9)
        .spacing(5),
    )
    .style(style::nav_container)
    .into()
}
