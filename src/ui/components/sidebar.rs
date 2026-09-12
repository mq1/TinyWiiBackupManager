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
    let import_queue_button: Element<'_, Message> = {
        if !state.import_queue.is_empty() {
            let icon = if state.ongoing.contains(Ongoing::AnimationState) {
                Icon::ArrowUp10
            } else {
                Icon::ArrowUp01
            };

            tooltip(
                my_sidebar_button([icon], state.current_page == Page::ImportQueue)
                    .on_press(Message::NavigateTo(Page::ImportQueue)),
                my_card("Import queue"),
                tooltip::Position::Right,
            )
            .into()
        } else {
            space().into()
        }
    };

    column![
        tooltip(
            my_sidebar_button([Icon::Gamepad2], state.current_page == Page::Games)
                .on_press(Message::NavigateTo(Page::Games)),
            my_card("Games"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button(
                [Icon::Waves, Icon::Bubbles],
                state.current_page == Page::HomebrewApps
            )
            .on_press(Message::NavigateTo(Page::HomebrewApps)),
            my_card("Homebrew Apps"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button([Icon::Waves, Icon::ArrowBigDown], false),
            my_card("Open Shop Channel"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button([Icon::ToolCase], state.current_page == Page::Toolbox)
                .on_press(Message::NavigateTo(Page::Toolbox)),
            my_card("Toolbox"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button([Icon::Settings], state.current_page == Page::Settings)
                .on_press(Message::NavigateTo(Page::Settings)),
            my_card("Settings"),
            tooltip::Position::Right
        ),
        space::vertical(),
        import_queue_button,
        tooltip(
            my_sidebar_button([Icon::HardDrive], false).on_press(Message::PickMountPoint),
            my_card("Select a drive"),
            tooltip::Position::Right
        ),
        tooltip(
            my_sidebar_button([Icon::Info], state.current_page == Page::About)
                .on_press(Message::NavigateTo(Page::About)),
            my_card("About"),
            tooltip::Position::Right
        )
    ]
    .padding(10)
    .spacing(10)
    .into()
}
