// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::{components, pages::Page},
};
use iced::{
    Element,
    widget::{column, space, tooltip},
};
use lucide_icons::Icon;

pub fn view(state: &AppState) -> Element<'_, Message> {
    column![
        components::sidebar_button::view(&[Icon::Gamepad2], state.current_page == Page::Games)
            .on_press(Message::NavigateTo(Page::Games)),
        components::sidebar_button::view(
            &[Icon::Waves, Icon::Bubbles],
            state.current_page == Page::HomebrewApps
        )
        .on_press(Message::NavigateTo(Page::HomebrewApps)),
        components::sidebar_button::view(&[Icon::Waves, Icon::ArrowBigDown], false),
        components::sidebar_button::view(&[Icon::ToolCase], state.current_page == Page::Toolbox)
            .on_press(Message::NavigateTo(Page::Toolbox)),
        components::sidebar_button::view(&[Icon::Settings], state.current_page == Page::Settings)
            .on_press(Message::NavigateTo(Page::Settings)),
        space::vertical(),
        tooltip(
            components::sidebar_button::view(&[Icon::HardDrive], false)
                .on_press(Message::PickMountPoint),
            components::card::view("Select a drive"),
            tooltip::Position::Right
        ),
        components::sidebar_button::view(&[Icon::Info], state.current_page == Page::About)
            .on_press(Message::NavigateTo(Page::About)),
    ]
    .padding(10)
    .spacing(10)
    .into()
}
