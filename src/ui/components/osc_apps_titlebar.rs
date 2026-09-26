// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    osc::osc_state::OscAppList,
    state::AppState,
    ui::components::{
        osc_app_search::osc_app_search, refresh_osc_button::refresh_osc_button, view_as::view_as,
        wii_ip::wii_ip,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space},
};

pub fn osc_apps_titlebar<'a>(apps: &'a OscAppList, state: &'a AppState) -> Element<'a, Message> {
    row![
        osc_app_search(apps),
        space::horizontal(),
        wii_ip(state),
        space().width(5),
        view_as(state),
        space().width(5),
        refresh_osc_button(state),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .padding(10)
    .into()
}
