// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::QueuedImport, ui::components::row_button::row_button};
use iced::{
    Element, Length, padding,
    widget::{checkbox, column, row, space, text},
};
use lucide_icons::Icon;

pub fn queued_import_row(
    row: &QueuedImport,
    busy: bool,
    in_current_batch: bool,
) -> Element<'_, Message> {
    let id = row.id;
    let entry = &row.entry;
    let mut details = column![
        text(format!(
            "{} [{}]",
            entry.meta().game_title(),
            entry.meta().game_id()
        )),
        text(entry.path().to_string_lossy()).size(12),
    ]
    .spacing(3);
    if let Some(error) = &row.error {
        details = details.push(text(format!("Failed: {error}")).size(12));
    } else if busy {
        details = details.push(
            text(if in_current_batch {
                "Current batch"
            } else {
                "Next batch"
            })
            .size(12),
        );
    }
    row![
        checkbox(row.selected).on_toggle_maybe(
            (!busy).then_some(move |selected| Message::ToggleImportGame(id, selected))
        ),
        details,
        space::horizontal(),
        row_button(
            Icon::X,
            "Remove from queue",
            (!busy).then_some(move || Message::CancelImport(id))
        )
    ]
    .spacing(5)
    .height(Length::Shrink)
    .padding(padding::all(8))
    .into()
}
