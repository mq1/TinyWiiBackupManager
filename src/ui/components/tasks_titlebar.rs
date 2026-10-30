// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::components::my_button::{MyButtonKind, my_button},
};
use iced::{
    Alignment, Element,
    widget::{row, space},
};
use lucide_icons::Icon;

pub fn tasks_titlebar(state: &AppState) -> Element<'_, Message> {
    let mut cancel_all_btn = my_button()
        .label("Cancel all")
        .icon(Icon::X)
        .kind(MyButtonKind::Secondary);

    if state.tasks.iter().any(|t| !t.state().is_finished()) {
        cancel_all_btn = cancel_all_btn.on_press(Message::CancelAllTasks);
    }

    let mut clear_btn = my_button()
        .label("Clear finished")
        .icon(Icon::BrushCleaning)
        .kind(MyButtonKind::Secondary);

    if state.tasks.iter().any(|t| t.state().is_finished()) {
        clear_btn = clear_btn.on_press(Message::ClearTaskHistory);
    }

    row!["  Tasks", space::horizontal(), clear_btn, cancel_all_btn]
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center)
        .into()
}
