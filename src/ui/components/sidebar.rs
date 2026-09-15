// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::{AppState, Ongoing},
    ui::{
        components::{my_card::my_card, my_sidebar_button::my_sidebar_button},
        pages::Page,
    },
};
use iced::{
    Element,
    widget::{column, space, tooltip},
};
use lucide_icons::Icon;

pub fn sidebar(state: &AppState) -> Element<'_, Message> {
    column![
        tooltip(
            my_sidebar_button(&[Icon::Gamepad2])
                .active(state.current_page == Page::Games)
                .on_press(Message::NavigateTo(Page::Games)),
            my_card("Games"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(&[Icon::Waves, Icon::Bubbles])
                .active(state.current_page == Page::HomebrewApps)
                .on_press(Message::NavigateTo(Page::HomebrewApps)),
            my_card("Homebrew Apps"),
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
        (!state.import_queue.is_empty()).then(|| {
            let import_queue_icon = if state.ongoing.contains(Ongoing::AnimationState) {
                Icon::ArrowUp10
            } else {
                Icon::ArrowUp01
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
            my_sidebar_button(&[Icon::HardDrive]).on_press(Message::PickMountPoint),
            my_card("Select a drive"),
            tooltip::Position::Right
        ),
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
