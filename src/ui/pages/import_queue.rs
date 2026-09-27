// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::components::{
        import_queue_titlebar::import_queue_titlebar, my_card::my_card,
        queued_import_row::queued_import_row,
    },
};
use iced::{
    Element, Length, padding,
    widget::{Column, column, container, rule, scrollable},
};
use itertools::Itertools;
use tap::Pipe;

pub fn import_queue(state: &AppState) -> Element<'_, Message> {
    let content = state
        .import_queue
        .iter()
        .enumerate()
        .map(queued_import_row)
        .intersperse_with(|| rule::horizontal(1).into())
        .collect::<Column<'_, _>>()
        .pipe(my_card)
        .padding(0);

    column![
        import_queue_titlebar(state),
        scrollable(container(content).padding(padding::all(10).right(20).top(0)))
            .width(Length::Fill),
    ]
    .into()
}
