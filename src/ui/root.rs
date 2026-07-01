// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::{
        components, my_palette,
        pages::{self, Page},
    },
};
use iced::{
    Element, Length, border,
    widget::{container, row, stack},
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    container(stack![
        row![
            components::sidebar::view(state),
            container(match state.current_page {
                Page::Games => pages::games::view(),
                Page::Settings => pages::settings::view(),
                Page::Toolbox => pages::toolbox::view(state),
                Page::Plugins => pages::plugins::view(state),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme| {
                let mut base = container::bordered_box(theme);
                base.border.radius = border::radius(10);
                base
            })
        ],
        container(components::notifications::view(state))
            .align_right(Length::Fill)
            .align_bottom(Length::Fill)
    ])
    .style(|theme| container::background(my_palette::card_bg(theme)))
    .into()
}
