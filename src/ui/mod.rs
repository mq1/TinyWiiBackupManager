// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    hbc::{app::HbcApp, osc::OscAppMeta},
    message::Message,
    state::State,
};
use iced::{
    Element, Length, padding,
    widget::{Column, column, container, row, rule, stack, text},
};

mod components;
pub mod dialogs;
pub mod lucide;
mod style;
pub use components::message_dialog::MyMessageDialog;

#[cfg(feature = "win10")]
pub mod window_color;

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

    let root: Element<'_, Message> = if cfg!(any(feature = "macos", feature = "win10")) {
        stack![
            container(row![].width(Length::Fill).height(Length::Fill)).style(style::nav_container),
            container(col).style(style::root_container)
        ]
        .into()
    } else {
        container(col).style(style::root_container).into()
    };

    let root = row![components::nav::view(state), root];

    let mut stack = stack![root];
    if let Some(message_dialog) = &state.message_dialog {
        stack = stack.push(message_dialog.view());
    }
    if !state.notifications.is_empty() {
        stack = stack.push(components::notifications::view(state));
    }

    if cfg!(target_os = "macos") {
        column![
            container(row![].width(Length::Fill).height(32)).style(style::nav_container),
            stack
        ]
        .into()
    } else {
        stack.into()
    }
}
