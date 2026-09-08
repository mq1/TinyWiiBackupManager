// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, state::AppState, ui::components::homebrew_app_search::homebrew_app_search,
};
use iced::{
    Alignment, Element,
    widget::{Text, row, space, text},
};

fn usage(state: &AppState) -> Text<'_> {
    text!(
        "{} app{}   ~ {}  ",
        state.homebrew_apps.count(),
        if state.homebrew_apps.count() == 1 {
            ""
        } else {
            "s"
        },
        state.homebrew_apps.total_size()
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
