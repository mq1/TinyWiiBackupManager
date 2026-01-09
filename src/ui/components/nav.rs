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
use lucide_icons::iced::{
    icon_arrow_down_0_1, icon_arrow_down_1_0, icon_gamepad_2, icon_hard_drive, icon_info,
    icon_moon, icon_settings, icon_shopping_bag, icon_sun, icon_sun_moon, icon_tool_case,
    icon_waves,
};

pub fn view(state: &State) -> Element<'_, Message> {
    let theme_icon = match state.config.get_theme_pref() {
        ThemePreference::Light => icon_sun(),
        ThemePreference::Dark => icon_moon(),
        ThemePreference::System => icon_sun_moon(),
    };

    let transfer_button: Element<'_, Message> = if state.transfer_stack.is_empty() {
        space().into()
    } else {
        let icon = if state.half_sec_anim_state {
            icon_arrow_down_0_1()
        } else {
            icon_arrow_down_1_0()
        };

        button(icon.size(20).center())
            .style(style::inactive_nav_button)
            .height(40)
            .width(40)
            .into()
    };

    container(
        column![
            button(icon_gamepad_2().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::Games))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Games)),
            button(icon_waves().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::HbcApps))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::HbcApps)),
            button(icon_shopping_bag().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::Osc))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Osc)),
            button(icon_tool_case().size(20).center())
                .style(style::inactive_nav_button)
                .height(40)
                .width(40),
            button(icon_settings().size(20).center())
                .style(style::get_nav_button_style(
                    state.screen == Screen::Settings
                ))
                .height(40)
                .width(40)
                .on_press(Message::NavigateTo(Screen::Settings)),
            space::vertical(),
            transfer_button,
            my_tooltip::view(
                button(icon_hard_drive().size(20).center())
                    .style(style::inactive_nav_button)
                    .height(40)
                    .width(40)
                    .on_press(Message::SelectMountPoint),
                "Select Drive/Mount Point"
            ),
            my_tooltip::view(
                button(theme_icon.size(20).center())
                    .style(style::inactive_nav_button)
                    .height(40)
                    .width(40)
                    .on_press(Message::ChangeTheme),
                "Change Theme (System/Light/Dark)"
            ),
            button(icon_info().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::About))
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
