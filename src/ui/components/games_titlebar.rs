// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::view_as};
use iced::{
    Element,
    widget::{row, space, text},
};
use lucide_icons::iced::icon_chevron_right;

pub fn view(state: &AppState) -> Element<'_, Message> {
    row![
        icon_chevron_right().size(20),
        text("Games").size(20),
        space::horizontal(),
        view_as::view(state)
    ]
    .spacing(5)
    .into()
}
