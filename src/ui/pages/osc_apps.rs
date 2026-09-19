// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    messages::Message,
    osc::osc_state::OscState,
    state::AppState,
    ui::{
        components::{
            my_card::my_card, osc_app_card::osc_app_card, osc_app_row::osc_app_row,
            osc_apps_other_toolbar::osc_apps_other_toolbar, osc_apps_titlebar::osc_apps_titlebar,
        },
        pages::{errored::errored, loading::loading},
    },
};
use iced::{
    Element,
    widget::{Column, Row, column, rule},
};
use itertools::Itertools;
use tap::Pipe;

pub fn osc_apps(state: &AppState) -> Element<'_, Message> {
    match &state.osc_contents {
        OscState::NotLoaded | OscState::Loading => loading(),
        OscState::Errored(e) => errored(e),
        OscState::Loaded(apps) => {
            let content: Element<'_, Message> = match state.config.view_as {
                ViewAs::Grid => apps
                    .iter()
                    .map(osc_app_card)
                    .collect::<Row<'_, _>>()
                    .spacing(10)
                    .wrap()
                    .into(),

                ViewAs::Table => apps
                    .iter()
                    .map(osc_app_row)
                    .intersperse_with(|| rule::horizontal(1).into())
                    .collect::<Column<'_, _>>()
                    .pipe(my_card)
                    .padding(0)
                    .into(),
            };

            column![
                osc_apps_titlebar(state),
                osc_apps_other_toolbar(apps),
                content
            ]
            .padding(10)
            .spacing(10)
            .into()
        }
    }
}
