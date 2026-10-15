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
use lucide_iced::bytes::{
    ARROW_BIG_DOWN, ARROW_UP_0_1, ARROW_UP_1_0, BELL_PLUS, BUBBLES, GAMEPAD_2, HARD_DRIVE, INFO,
    SETTINGS, TOOL_CASE, WAVES_HORIZONTAL,
};

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    column![
        tooltip(
            my_sidebar_button(&[GAMEPAD_2])
                .active(state.current_page == Page::Games)
                .on_press(Message::NavigateTo(Page::Games)),
            my_card("Games"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[WAVES_HORIZONTAL, BUBBLES])
                .active(state.current_page == Page::HomebrewApps)
                .on_press(Message::NavigateTo(Page::HomebrewApps)),
            my_card("Homebrew Apps"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[WAVES_HORIZONTAL, ARROW_BIG_DOWN])
                .active(state.current_page == Page::Osc)
                .on_press(Message::NavigateTo(Page::Osc)),
            my_card("Open Shop Channel"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[TOOL_CASE])
                .active(state.current_page == Page::Toolbox)
                .on_press(Message::NavigateTo(Page::Toolbox)),
            my_card("Toolbox"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[SETTINGS])
                .active(state.current_page == Page::Settings)
                .on_press(Message::NavigateTo(Page::Settings)),
            my_card("Settings"),
            tooltip::Position::Right
        ),
        space::vertical(),
        state.new_version.as_ref().map(|new_version| {
            tooltip(
                my_sidebar_button(&[BELL_PLUS]).on_press_with(|| {
                    Message::Open(
                        "https://github.com/mq1/TinyWiiBackupManager/releases/latest".into(),
                    )
                }),
                my_card(text!(
                    "New TinyWiiBackupManager version available: {new_version}"
                )),
                tooltip::Position::Right,
            )
        }),
        (!state.import_queue.is_empty()).then(|| {
            let import_queue_icon = if state.animation_state {
                ARROW_UP_1_0
            } else {
                ARROW_UP_0_1
            };

            tooltip(
                my_sidebar_button(&[import_queue_icon])
                    .active(state.current_page == Page::ImportQueue)
                    .on_press(Message::NavigateTo(Page::ImportQueue)),
                my_card("Import queue"),
                tooltip::Position::Right,
            )
        }),
        tooltip(
            my_sidebar_button(&[HARD_DRIVE]).on_press(Message::PickMountPoint),
            my_card("Select a drive"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[INFO])
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
