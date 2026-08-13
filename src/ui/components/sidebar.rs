// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::{
        components::{my_card::MyCard, my_sidebar_button::MySidebarButton},
        pages::Page,
    },
};
use iced::{
    Element,
    widget::{column, space, tooltip},
};
use lucide_icons::Icon;

pub fn view(state: &AppState) -> Element<'_, Message> {
    column![
        tooltip(
            MySidebarButton::new(&[Icon::Gamepad2])
                .active_if(state.current_page == Page::Games)
                .view()
                .on_press(Message::NavigateTo(Page::Games)),
            MyCard::new("Games").view(),
            tooltip::Position::Right
        ),
        tooltip(
            MySidebarButton::new(&[Icon::Waves, Icon::Bubbles])
                .active_if(state.current_page == Page::HomebrewApps)
                .view()
                .on_press(Message::NavigateTo(Page::HomebrewApps)),
            MyCard::new("Homebrew Apps").view(),
            tooltip::Position::Right
        ),
        tooltip(
            MySidebarButton::new(&[Icon::Waves, Icon::ArrowBigDown]).view(),
            MyCard::new("Open Shop Channel").view(),
            tooltip::Position::Right
        ),
        tooltip(
            MySidebarButton::new(&[Icon::ToolCase])
                .active_if(state.current_page == Page::Toolbox)
                .view()
                .on_press(Message::NavigateTo(Page::Toolbox)),
            MyCard::new("Toolbox").view(),
            tooltip::Position::Right
        ),
        tooltip(
            MySidebarButton::new(&[Icon::Settings])
                .active_if(state.current_page == Page::Settings)
                .view()
                .on_press(Message::NavigateTo(Page::Settings)),
            MyCard::new("Settings").view(),
            tooltip::Position::Right
        ),
        space::vertical(),
        tooltip(
            MySidebarButton::new(&[Icon::HardDrive])
                .view()
                .on_press(Message::PickMountPoint),
            MyCard::new("Select a drive").view(),
            tooltip::Position::Right
        ),
        tooltip(
            MySidebarButton::new(&[Icon::Info])
                .active_if(state.current_page == Page::About)
                .view()
                .on_press(Message::NavigateTo(Page::About)),
            MyCard::new("About").view(),
            tooltip::Position::Right
        )
    ]
    .padding(10)
    .spacing(10)
    .into()
}
