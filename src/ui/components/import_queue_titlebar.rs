// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ui::fast_transfer::fast_transfer,
    message::Message,
    state::AppState,
    ui::components::my_button::{MyButtonKind, my_button},
    util::drive_state::DriveState,
};
use iced::{
    Alignment, Element,
    widget::{checkbox, column, pick_list, row, space, text},
};
use lucide_icons::Icon;

pub fn import_queue_titlebar(state: &AppState) -> Element<'_, Message> {
    let busy = state.is_import_busy();
    let cancelling = state.is_import_cancelling();
    let selected_count = state.selected_import_count();
    let mut import_btn = my_button()
        .label(if busy {
            "Importing..."
        } else {
            "Import selected"
        })
        .icon(Icon::Play)
        .kind(MyButtonKind::Primary);

    if !busy
        && selected_count > 0
        && !state.config.mount_point().as_os_str().is_empty()
        && matches!(state.drive, DriveState::Loaded(_))
    {
        import_btn = import_btn.on_press(Message::TriggerImport);
    }
    if let Some(run) = &state.active_import {
        import_btn = my_button()
            .label(if cancelling {
                "Cancelling..."
            } else {
                "Cancel import"
            })
            .icon(Icon::X)
            .kind(MyButtonKind::Danger);
        if !cancelling {
            import_btn = import_btn.on_press(Message::CancelImportRun(run.id));
        }
    }

    let mut remove_all_btn = my_button()
        .label("Remove all")
        .icon(Icon::X)
        .kind(MyButtonKind::Secondary);

    if !busy && !state.import_queue.is_empty() {
        remove_all_btn = remove_all_btn.on_press(Message::CancelAllImports);
    }
    let jobs: Element<'_, Message> = if busy {
        text(state.import_parallel_jobs.to_string()).into()
    } else {
        pick_list(
            [1, 2, 3, 4],
            Some(state.import_parallel_jobs),
            Message::SetImportParallelJobs,
        )
        .into()
    };
    let destination = state.active_import.as_ref().map_or_else(
        || state.config.mount_point(),
        |run| run.mount_point.as_path(),
    );

    column![
        row![
            "Detected games to import",
            space::horizontal(),
            remove_all_btn,
            import_btn
        ].spacing(10).align_y(Alignment::Center),
        row![
            checkbox(selected_count > 0 && selected_count == state.import_queue.len())
                .label("Select all")
                .on_toggle_maybe((!busy).then_some(Message::SelectAllImportGames)),
            text(format!("{selected_count} of {} selected", state.import_queue.len())),
            space::horizontal(),
            text("Simultaneous games"),
            jobs,
        ].spacing(10).align_y(Alignment::Center),
        text(format!("Destination: {}", destination.display())).size(12),
        fast_transfer(state.config.fast_transfer(), state.can_change_fast_transfer()).map(Message::Config),
        text(if cancelling {
            "Stopping imports and cleaning up incomplete files. Completed games are kept."
        } else if busy {
            "Newly added games wait for the next batch. Failed games remain available to retry."
        } else {
            "1 imports games sequentially; 2–4 import games in parallel. More jobs may be slower on a USB drive."
        }).size(12),
    ]
    .spacing(10)
    .padding(10)
    .into()
}
