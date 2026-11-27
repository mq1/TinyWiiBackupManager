// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::{
        components::{
            homebrew_app_card::homebrew_app_card, homebrew_app_row::homebrew_app_row,
            homebrew_apps_titlebar::homebrew_apps_titlebar, my_card::my_card,
        },
        pages::{errored::errored, loading::loading, no_drive::no_drive},
    },
};
use iced::{
    Element, Length, padding,
    widget::{Column, Row, column, container, rule, scrollable},
};
use itertools::Itertools;
use twbm_common::config_contents::ViewAs;
use twbm_homebrew::manager::HomebrewAppManager;

pub fn homebrew_apps(state: &AppState) -> Element<'_, Message> {
    if state.config.contents.mount_point.is_none() {
        return no_drive();
    }

    match &state.homebrew {
        HomebrewAppManager::Loading => loading(),
        HomebrewAppManager::Errored(e) => errored(e),
        HomebrewAppManager::Loaded(apps) => {
            let content: Element<'_, Message> = match state.config.contents.view_as {
                ViewAs::Grid => apps
                    .iter_by(state.config.contents.sort_by)
                    .map(|app| {
                        homebrew_app_card(app, state.local_to_osc_app_map.get(app.dirname()))
                    })
                    .collect::<Row<'_, _>>()
                    .spacing(10)
                    .wrap()
                    .into(),

                ViewAs::Table => my_card(
                    apps.iter_by(state.config.contents.sort_by)
                        .map(|app| {
                            homebrew_app_row(app, state.local_to_osc_app_map.get(app.dirname()))
                        })
                        .intersperse_with(|| rule::horizontal(1).into())
                        .collect::<Column<'_, _>>(),
                )
                .padding(0)
                .into(),
            };

            column![
                homebrew_apps_titlebar(apps, state),
                scrollable(container(content).padding(padding::all(10).right(20).top(0)))
                    .width(Length::Fill),
            ]
            .into()
        }
    }
}
