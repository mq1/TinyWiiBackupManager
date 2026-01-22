// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Element,
    widget::{container, row, rule, stack},
};

mod components;
pub mod dialogs;
pub mod lucide;
mod style;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    Games,
    GameInfo(usize),
    HbcApps,
    HbcInfo(usize),
    Osc,
    OscInfo(usize),
    Toolbox,
    Settings,
    Transfer,
    About,
}

pub fn view(state: &State) -> Element<'_, Message> {
    let root = container(row![
        components::nav::view(state),
        rule::vertical(1),
        match state.screen {
            Screen::Games => components::games::view(state),
            Screen::GameInfo(i) =>
                components::game_info::view(state, state.game_list.get_unchecked(i)),
            Screen::HbcApps => components::hbc::view(state),
            Screen::HbcInfo(hbc_i) => components::hbc_info::view(state, hbc_i),
            Screen::Osc => components::osc::view(state),
            Screen::OscInfo(osc_i) => components::osc_info::view(state, osc_i),
            Screen::Toolbox => components::toolbox::view(state),
            Screen::Settings => components::settings::view(state),
            Screen::Transfer => components::transfer::view(state),
            Screen::About => components::about::view(state),
        },
    ])
    .style(style::root_container);

    if state.notifications.is_empty() {
        root.into()
    } else {
        stack![root, components::notifications::view(state)].into()
    }
}
