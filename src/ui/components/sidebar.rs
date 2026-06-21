// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::{components, pages::Page},
};
use iced::{
    Element,
    widget::{column, space},
};
use lucide_icons::Icon;

pub fn view<'a>(state: &AppState) -> Element<'a, Message> {
    column![
        components::sidebar_button::view(&[Icon::Gamepad2], state.current_page() == Page::Games)
            .on_press(Message::NavigateTo(Page::Games)),
        components::sidebar_button::view(&[Icon::Waves, Icon::Bubbles], false),
        components::sidebar_button::view(&[Icon::Waves, Icon::ArrowBigDown], false),
        components::sidebar_button::view(&[Icon::ToolCase], false),
        components::sidebar_button::view(&[Icon::Settings], state.current_page() == Page::Settings)
            .on_press(Message::NavigateTo(Page::Settings)),
        space::vertical(),
        components::sidebar_button::view(&[Icon::Info], false),
    ]
    .padding(10)
    .spacing(10)
    .into()
}
