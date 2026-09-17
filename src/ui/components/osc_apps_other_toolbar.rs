// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, osc::osc_contents::OscAppList,
    ui::components::osc_app_search::osc_app_search,
};
use humantime::format_duration;
use iced::{
    Alignment, Element, padding,
    widget::{Text, row, space, text},
};

fn last_refresh(apps: &OscAppList) -> Text<'_> {
    text!("refreshed {} ago", format_duration(apps.last_refresh()))
}

pub fn osc_apps_other_toolbar(apps: &OscAppList) -> Element<'_, Message> {
    row![
        osc_app_search(apps),
        space::horizontal(),
        last_refresh(apps)
    ]
    .align_y(Alignment::Center)
    .padding(padding::all(10).top(0).right(20))
    .into()
}
