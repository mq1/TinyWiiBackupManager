// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, state::AppState, ui::components::homebrew_app_search::homebrew_app_search,
    util::drive_state::DriveState,
};
use iced::{
    Alignment, Element,
    widget::{Text, row, space, text},
};

fn usage(state: &AppState) -> Text<'_> {
    text!(
        "{} app{}   ~ {}  ",
        state.homebrew.count(),
        if state.homebrew.count() == 1 { "" } else { "s" },
        if let DriveState::Loaded(info) = &state.drive {
            info.apps_size_str()
        } else {
            ""
        }
    )
}

pub fn homebrew_apps_other_toolbar(state: &AppState) -> Element<'_, Message> {
    row![
        homebrew_app_search(state),
        space::horizontal(),
        usage(state)
    ]
    .align_y(Alignment::Center)
    .into()
}
