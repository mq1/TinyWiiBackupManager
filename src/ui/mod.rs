// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

mod components;
pub mod dialogs;
pub mod lucide;
mod style;

#[cfg(target_vendor = "pc")]
pub mod window_color;

#[cfg(target_vendor = "win7")]
pub mod xp_dialogs;

use crate::{
    games::game::Game,
    hbc::{app::HbcApp, osc::OscAppMeta},
    message::Message,
    state::State,
};
use iced::{
    Element, Length, padding,
    widget::{Column, Stack, column, container, row, rule, text},
};

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
    if let Some((ref title, ref description, level, ref callback)) = state.message_box {
        return components::message_box::view(title, description, level, callback.as_deref());
    }

    let mut col = Column::new();

    col = col.push(match &state.screen {
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
    });

    if !state.status.is_empty() {
        col = col
            .push(rule::horizontal(1))
            .push(container(text(&state.status)).padding(padding::horizontal(10).vertical(5)));
    }

    let mut stack = Stack::new();
    if cfg!(any(target_os = "macos", target_vendor = "pc")) {
        stack = stack.push(
            container(row![].width(Length::Fill).height(Length::Fill)).style(style::nav_container),
        );
    }
    stack = stack.push(container(col).style(style::root_container));
    if !state.notifications.is_empty() {
        stack = stack.push(components::notifications::view(state));
    }

    let root = row![components::nav::view(state), stack];

    if cfg!(target_os = "macos") {
        column![
            container(row![].width(Length::Fill).height(32)).style(style::nav_container),
            root
        ]
        .into()
    } else {
        root.into()
    }
}
