// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{refresh_osc_button::refresh_osc_button, view_as::view_as},
};
use iced::{
    Alignment, Element, padding,
    widget::{row, space, text},
};
use lucide_icons::iced::icon_chevron_right;

pub fn osc_apps_titlebar(state: &AppState) -> Element<'_, Message> {
    row![
        icon_chevron_right().size(20),
        text("Open Shop Channel (oscwii.org)").size(20),
        space::horizontal(),
        view_as(state),
        space().width(5),
        refresh_osc_button(state),
    ]
    .spacing(5)
    .align_y(Alignment::Center)
    .padding(padding::all(10).bottom(0))
    .into()
}
