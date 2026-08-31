// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_card::my_card, queued_import_row::queued_import_row},
};
use iced::{
    Element, padding,
    widget::{Column, column, row, rule, text},
};
use itertools::Itertools;
use lucide_icons::iced::icon_chevron_right;

pub fn import_queue(state: &AppState) -> Element<'_, Message> {
    let titlebar = row![icon_chevron_right().size(20), text("Import Queue").size(20)]
        .spacing(5)
        .padding(padding::all(14).left(10));

    let content = state
        .import_queue
        .iter()
        .enumerate()
        .map(queued_import_row)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>();

    column![titlebar, my_card(content).padding(0)]
        .padding(10)
        .spacing(10)
        .into()
}
