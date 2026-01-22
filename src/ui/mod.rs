// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    hbc::{app::HbcApp, osc::OscAppMeta},
    message::Message,
    state::State,
};
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
    GameInfo(Game),
    HbcApps,
    HbcInfo(HbcApp),
    Osc,
    OscInfo(OscAppMeta),
    Toolbox,
    Settings,
    Transfer,
    About,
}

pub fn view(state: &State) -> Element<'_, Message> {
    let root = container(row![
        components::nav::view(state),
        rule::vertical(1),
        match &state.screen {
            Screen::Games => components::games::view(state),
            Screen::GameInfo(game) => components::game_info::view(state, game),
            Screen::HbcApps => components::hbc::view(state),
            Screen::HbcInfo(app) => components::hbc_info::view(state, app),
            Screen::Osc => components::osc::view(state),
            Screen::OscInfo(app) => components::osc_info::view(state, app),
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
