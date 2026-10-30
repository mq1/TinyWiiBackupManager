// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::{
        components::{my_card::my_card, my_sidebar_button::my_sidebar_button},
        pages::Page,
    },
};
use iced::{
    Element,
    widget::{column, space, text, tooltip},
};
use lucide_icons::Icon;

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    column![
        tooltip(
            my_sidebar_button(&[Icon::Gamepad2])
                .active(state.current_page == Page::Library)
                .on_press(Message::NavigateTo(Page::Library)),
            my_card("Games"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[Icon::MonitorSmartphone])
                .active(matches!(
                    state.current_page,
                    // a drive's pages belong to My Consoles
                    Page::Consoles | Page::Games | Page::HomebrewApps
                ))
                .on_press(Message::NavigateTo(Page::Consoles)),
            my_card("My Consoles"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[Icon::Waves, Icon::ArrowBigDown])
                .active(state.current_page == Page::Osc)
                .on_press(Message::NavigateTo(Page::Osc)),
            my_card("Open Shop Channel"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[Icon::ToolCase])
                .active(state.current_page == Page::Toolbox)
                .on_press(Message::NavigateTo(Page::Toolbox)),
            my_card("Toolbox"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[Icon::Settings])
                .active(state.current_page == Page::Settings)
                .on_press(Message::NavigateTo(Page::Settings)),
            my_card("Settings"),
            tooltip::Position::Right
        ),
        space::vertical(),
        state.new_version.as_ref().map(|new_version| {
            tooltip(
                my_sidebar_button(&[Icon::BellPlus]).on_press_with(|| {
                    Message::Open(
                        "https://github.com/petter1/UltimateGameBackupManager/releases/latest"
                            .into(),
                    )
                }),
                my_card(text!(
                    "New Ultimate Game Backup Manager version available: {new_version}"
                )),
                tooltip::Position::Right,
            )
        }),
        state.tasks.iter().next().is_some().then(|| {
            let tasks_icon = if state.animation_state {
                Icon::ArrowUp10
            } else {
                Icon::ArrowUp01
            };

            tooltip(
                my_sidebar_button(&[tasks_icon])
                    .active(state.current_page == Page::Tasks)
                    .on_press(Message::NavigateTo(Page::Tasks)),
                my_card("Tasks"),
                tooltip::Position::Right,
            )
        }),
        tooltip(
            my_sidebar_button(&[Icon::Info])
                .active(state.current_page == Page::About)
                .on_press(Message::NavigateTo(Page::About)),
            my_card("About"),
            tooltip::Position::Right
        )
    ]
    .padding(10)
    .spacing(10)
    .into()
}
