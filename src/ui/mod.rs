// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Element,
    widget::{container, row, rule, stack},
};

mod components;
pub mod dialogs;
mod style;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Screen {
    Games,
    HbcApps,
    Osc,
    About,
}

pub fn view(state: &State) -> Element<'_, Message> {
    stack![
        container(row![
            components::nav::view(state),
            rule::vertical(1),
            match state.screen {
                Screen::Games => components::games::view(state),
                Screen::HbcApps => components::hbc_apps::view(state),
                Screen::Osc => components::osc::view(state),
                Screen::About => components::about::view(),
            },
        ])
        .style(style::root_container),
        components::notifications::view(state)
    ]
    .into()
}
