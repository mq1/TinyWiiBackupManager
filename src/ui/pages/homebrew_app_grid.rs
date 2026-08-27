// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{
        homebrew_app_card::homebrew_app_card, homebrew_apps_titlebar::homebrew_apps_titlebar,
    },
};
use iced::{
    Element,
    widget::{Row, column},
};

pub fn homebrew_app_grid(state: &AppState) -> Element<'_, Message> {
    let content = state
        .homebrew_apps
        .iter_by(state.config.sort_by)
        .map(homebrew_app_card)
        .collect::<Row<'_, _>>()
        .spacing(10);

    column![homebrew_apps_titlebar(state), content]
        .padding(10)
        .spacing(10)
        .into()
}
