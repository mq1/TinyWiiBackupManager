// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    tasks::{Task, TaskState},
    ui::components::row_button::row_button,
};
use iced::{
    Element, Length, Theme, padding,
    widget::{column, progress_bar, row, space, text},
};
use lucide_icons::Icon;

pub fn task_row(task: &Task) -> Element<'_, Message> {
    let id = task.id();
    let progress = task.progress();

    let status: Element<'_, Message> = match task.state() {
        TaskState::Waiting => text("Waiting").into(),
        TaskState::Running | TaskState::Paused => {
            let mut line = progress.summary();
            if *task.state() == TaskState::Paused {
                line = format!("Paused · {line}");
            } else if !progress.message.is_empty() && progress.total_bytes.is_none() {
                line = progress.message.clone();
            }

            column![
                progress_bar(0.0..=1.0, progress.fraction().unwrap_or(0.0)).girth(6),
                text(line).size(12),
            ]
            .spacing(4)
            .into()
        }
        TaskState::Done(message) => text(message.as_str())
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().success),
            })
            .into(),
        TaskState::Failed(reason) => text(reason.as_str())
            .style(|theme: &Theme| text::Style {
                color: Some(theme.palette().danger),
            })
            .into(),
        TaskState::Cancelled => text("Cancelled").into(),
    };

    let buttons = match task.state() {
        TaskState::Waiting => row![row_button(
            Icon::X,
            "Cancel",
            Some(move || Message::CancelTask(id))
        )],
        TaskState::Running => row![
            row_button(Icon::Pause, "Pause", Some(move || Message::PauseTask(id))),
            row_button(Icon::X, "Cancel", Some(move || Message::CancelTask(id))),
        ],
        TaskState::Paused => row![
            row_button(Icon::Play, "Resume", Some(move || Message::ResumeTask(id))),
            row_button(Icon::X, "Cancel", Some(move || Message::CancelTask(id))),
        ],
        TaskState::Failed(_) | TaskState::Cancelled => row![row_button(
            Icon::RotateCcw,
            "Retry",
            Some(move || Message::RetryTask(id))
        )],
        TaskState::Done(_) => row![],
    }
    .spacing(5);

    row![
        column![text(task.title()), status]
            .spacing(4)
            .width(Length::Fill),
        space::horizontal().width(10),
        buttons,
    ]
    .padding(padding::all(6).left(10).right(10))
    .into()
}
