// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    messages::Message,
    state::AppState,
    ui::components::{
        homebrew_app_card::homebrew_app_card, homebrew_app_row::homebrew_app_row,
        homebrew_apps_other_toolbar::homebrew_apps_other_toolbar,
        homebrew_apps_titlebar::homebrew_apps_titlebar, my_card::my_card,
    },
};
use iced::{
    Element,
    widget::{Column, Row, column, rule},
};
use itertools::Itertools;
use tap::Pipe;

pub fn homebrew_apps(state: &AppState) -> Element<'_, Message> {
    let apps = state.homebrew.iter_by(state.config.sort_by);

    let content: Element<'_, Message> = match state.config.view_as {
        ViewAs::Grid => apps
            .map(homebrew_app_card)
            .collect::<Row<'_, _>>()
            .spacing(10)
            .wrap()
            .into(),

        ViewAs::Table => apps
            .map(homebrew_app_row)
            .intersperse_with(|| rule::horizontal(1).into())
            .collect::<Column<'_, _>>()
            .pipe(my_card)
            .padding(0)
            .into(),
    };

    column![
        homebrew_apps_titlebar(state),
        homebrew_apps_other_toolbar(state),
        content
    ]
    .padding(10)
    .spacing(10)
    .into()
}
