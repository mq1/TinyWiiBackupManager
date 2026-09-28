// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::conversion_state::ConversionState,
    message::Message,
    state::AppState,
    ui::components::my_button::{MyButtonKind, my_button},
};
use iced::{
    Element,
    widget::{row, space},
};
use lucide_icons::Icon;

pub fn import_queue_titlebar(state: &AppState) -> Element<'_, Message> {
    let mut import_btn = my_button()
        .label("Start importing")
        .icon(Icon::Play)
        .kind(MyButtonKind::Primary);

    if matches!(state.importing, ConversionState::Idle) && !state.import_queue.is_empty() {
        import_btn = import_btn.on_press(Message::TriggerImport);
    }

    let mut cancel_all_btn = my_button()
        .label("Cancel all")
        .icon(Icon::X)
        .kind(MyButtonKind::Secondary);

    if !state.import_queue.is_empty() {
        cancel_all_btn = cancel_all_btn.on_press(Message::CancelAllImports);
    }

    row![space::horizontal(), cancel_all_btn, import_btn]
        .padding(10)
        .into()
}
