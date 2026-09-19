// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    homebrew::homebrew_state::HomebrewState,
    messages::Message,
    state::AppState,
    ui::{
        components::{
            homebrew_app_card::homebrew_app_card, homebrew_app_row::homebrew_app_row,
            homebrew_apps_other_toolbar::homebrew_apps_other_toolbar,
            homebrew_apps_titlebar::homebrew_apps_titlebar, my_card::my_card,
        },
        pages::{errored::errored, loading::loading},
    },
};
use iced::{
    Element, Length, padding,
    widget::{Column, Row, column, rule, scrollable},
};
use itertools::Itertools;
use tap::Pipe;

pub fn homebrew_apps(state: &AppState) -> Element<'_, Message> {
    match &state.homebrew {
        HomebrewState::NotLoaded | HomebrewState::Loading => loading(),
        HomebrewState::Errored(e) => errored(e),
        HomebrewState::Loaded(apps) => {
            let content: Element<'_, Message> = match state.config.view_as {
                ViewAs::Grid => apps
                    .iter_by(state.config.sort_by)
                    .map(homebrew_app_card)
                    .collect::<Row<'_, _>>()
                    .spacing(10)
                    .wrap()
                    .into(),

                ViewAs::Table => apps
                    .iter_by(state.config.sort_by)
                    .map(homebrew_app_row)
                    .intersperse_with(|| rule::horizontal(1).into())
                    .collect::<Column<'_, _>>()
                    .pipe(my_card)
                    .padding(0)
                    .into(),
            };

            column![
                homebrew_apps_titlebar(state),
                scrollable(
                    column![homebrew_apps_other_toolbar(apps, &state.drive), content]
                        .padding(padding::all(10).right(20).top(0))
                        .spacing(10)
                )
                .width(Length::Fill),
            ]
            .into()
        }
    }
}
