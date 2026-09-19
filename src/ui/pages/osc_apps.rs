// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    osc::osc_state::OscAppList,
    state::AppState,
    ui::components::{
        osc_app_card::osc_app_card, osc_apps_other_toolbar::osc_apps_other_toolbar,
        osc_apps_titlebar::osc_apps_titlebar,
    },
};
use iced::{
    Element, Length, padding,
    widget::{Row, column, scrollable},
};

pub fn osc_apps<'a>(state: &'a AppState, apps: &'a OscAppList) -> Element<'a, Message> {
    let content = apps
        .iter()
        .map(osc_app_card)
        .collect::<Row<'_, _>>()
        .spacing(10)
        .padding(padding::left(10).bottom(10).right(20))
        .wrap();

    column![
        osc_apps_titlebar(state),
        scrollable(column![osc_apps_other_toolbar(apps), content]).width(Length::Fill)
    ]
    .spacing(10)
    .into()
}
