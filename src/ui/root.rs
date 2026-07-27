// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::{
        components::{self, Modal},
        my_palette,
        pages::{self, Page},
    },
};
use iced::{
    Color, Element, Length, border,
    widget::{Stack, container, opaque, row},
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content = Some(
        row![
            components::sidebar::view(state),
            container(match state.current_page {
                Page::Games => pages::game_grid::view(state),
                Page::Settings => pages::settings::view(),
                Page::Toolbox => pages::toolbox::view(state),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme| {
                let mut base = container::bordered_box(theme);
                base.border.radius = border::radius(10);
                base
            })
        ]
        .into(),
    );

    let modal = state.current_modal.as_ref().map(|modal| {
        opaque(
            container(match modal {
                Modal::GameInfo((idx, disc_info)) => {
                    components::game_info::view(&state.games[*idx], disc_info.as_deref())
                }
            })
            .center(Length::Fill)
            .style(|theme| container::transparent(theme).background(Color::BLACK.scale_alpha(0.7))),
        )
    });

    let notifications = (!state.notifications.is_empty()).then(|| {
        container(components::notifications::view(state))
            .align_right(Length::Fill)
            .align_bottom(Length::Fill)
            .into()
    });

    let stack = Stack::with_children([content, modal, notifications].into_iter().flatten());

    container(stack)
        .style(|theme| container::background(my_palette::card_bg(theme)))
        .into()
}
