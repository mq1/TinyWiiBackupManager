// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ThemePreference,
    message::Message,
    state::State,
    ui::{Screen, components::my_tooltip, style},
    updater::LATEST_VERSION_DOWNLOAD_URL,
};
use iced::{
    Element,
    widget::{Column, button, container, space},
};
use lucide_icons::iced::{
    icon_arrow_down_0_1, icon_cloud_backup, icon_gamepad_2, icon_hard_drive, icon_info, icon_moon,
    icon_settings, icon_store, icon_sun, icon_sun_moon, icon_tool_case, icon_waves,
};

pub fn view(state: &State) -> Element<'_, Message> {
    let theme_icon = match state.config.theme_preference() {
        ThemePreference::Light => icon_sun(),
        ThemePreference::Dark => icon_moon(),
        ThemePreference::System => icon_sun_moon(),
    };

    let mut col = Column::new()
        .padding(9)
        .spacing(5)
        .push(
            button(icon_gamepad_2().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::Games))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::Games)),
        )
        .push(
            button(icon_waves().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::HbcApps))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::HbcApps)),
        )
        .push(
            button(icon_store().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::Osc))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::Osc)),
        )
        .push(
            button(icon_tool_case().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::Toolbox))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::Toolbox)),
        )
        .push(
            button(icon_settings().size(20).center())
                .style(style::get_nav_button_style(
                    state.screen == Screen::Settings,
                ))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::Settings)),
        )
        .push(space::vertical());

    if state.transfer_queue.has_pending_operations() {
        col = col.push(
            button(icon_arrow_down_0_1().size(20).center())
                .style(style::get_nav_button_style(
                    state.screen == Screen::Transfer,
                ))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::Transfer)),
        );
    }

    if let Some(new_version) = &state.new_version {
        col = col.push(my_tooltip::view(
            button(icon_cloud_backup().size(20).center())
                .style(style::inactive_nav_button)
                .height(40)
                .width(40)
                .on_press_with(|| Message::OpenThat(LATEST_VERSION_DOWNLOAD_URL.into())),
            format!("New Version Available: {new_version}"),
        ));
    }

    col = col
        .push(my_tooltip::view(
            button(icon_hard_drive().size(20).center())
                .style(style::inactive_nav_button)
                .height(40)
                .width(40)
                .on_press(Message::SelectMountPoint),
            "Select Drive/Mount Point",
        ))
        .push(my_tooltip::view(
            button(theme_icon.size(20).center())
                .style(style::inactive_nav_button)
                .height(40)
                .width(40)
                .on_press(Message::ChangeTheme),
            "Change Theme (System/Light/Dark)",
        ))
        .push(
            button(icon_info().size(20).center())
                .style(style::get_nav_button_style(state.screen == Screen::About))
                .height(40)
                .width(40)
                .on_press(Message::NavTo(Screen::About)),
        );

    container(col).style(style::nav_container).into()
}
