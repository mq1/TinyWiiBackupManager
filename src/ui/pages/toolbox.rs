// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState, ui::components::drive_info::drive_info};
use iced::{
    Element, Length, padding,
    widget::{column, row, scrollable, text},
};
use lucide_icons::iced::icon_chevron_right;

pub fn toolbox(state: &AppState) -> Element<'_, Message> {
    let titlebar = row![icon_chevron_right().size(20), text("Toolbox").size(20)]
        .spacing(5)
        .padding(padding::all(14).left(10));

    let contents = scrollable(
        column![drive_info(state)]
            .spacing(10)
            .padding(padding::left(10).bottom(10).right(20))
            .width(Length::Fill),
    );

    column![titlebar, contents].into()
}
