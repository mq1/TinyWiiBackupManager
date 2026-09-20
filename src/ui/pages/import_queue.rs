// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{my_card::my_card, queued_import_row::queued_import_row},
};
use iced::{
    Element, Length, padding,
    widget::{Column, rule, scrollable},
};
use itertools::Itertools;
use tap::Pipe;

pub fn import_queue(state: &AppState) -> Element<'_, Message> {
    state
        .import_queue
        .iter()
        .enumerate()
        .map(queued_import_row)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>()
        .pipe(my_card)
        .padding(padding::all(10).right(20))
        .width(Length::Fill)
        .pipe(scrollable)
        .into()
}
