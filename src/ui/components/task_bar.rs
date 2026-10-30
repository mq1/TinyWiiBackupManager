// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::AppState, ui::pages::Page};
use iced::{
    Alignment, Element, Length, padding,
    widget::{button, progress_bar, row, text},
};

/// The bar at the bottom of every page while tasks run or wait; opens the task manager.
pub fn task_bar(state: &AppState) -> Option<Element<'_, Message>> {
    let running = state.tasks.running_count();
    let waiting = state.tasks.waiting_count();
    if running == 0 && waiting == 0 {
        return None;
    }

    let mut label = match running {
        1 => "1 task running".to_string(),
        n => format!("{n} tasks running"),
    };
    if waiting > 0 {
        label.push_str(&format!(" · {waiting} waiting"));
    }

    let fraction = state.tasks.overall_fraction();
    let percent = fraction.map(|f| format!("{:.0}%", f * 100.0));

    let content = row![
        text(label),
        progress_bar(0.0..=1.0, fraction.unwrap_or(0.0))
            .length(Length::Fill)
            .girth(6),
        percent.map(text),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    Some(
        button(content)
            .on_press(Message::NavigateTo(Page::Tasks))
            .style(button::subtle)
            .padding(padding::horizontal(10).vertical(4))
            .width(Length::Fill)
            .into(),
    )
}
