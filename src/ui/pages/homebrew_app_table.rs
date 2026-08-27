// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{
        homebrew_app_row::homebrew_app_row, homebrew_apps_titlebar::homebrew_apps_titlebar,
        my_card::my_card,
    },
};
use iced::{
    Element,
    widget::{Column, column, rule},
};
use itertools::Itertools;

pub fn homebrew_app_table(state: &AppState) -> Element<'_, Message> {
    let content = state
        .homebrew_apps
        .iter_by(state.config.sort_by)
        .map(homebrew_app_row)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>();

    column![homebrew_apps_titlebar(state), my_card(content).padding(0)]
        .padding(10)
        .spacing(10)
        .into()
}
