// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{AppWindow, Dispatcher, Message, UiState};
use slint::{ComponentHandle, SharedString, Weak};
use twbm_core::{config::Config, conversion_queue::QueuedConversion, drive_info::DriveInfo};

pub fn perform_conversion(
    conv: QueuedConversion,
    config: Config,
    drive_info: DriveInfo,
    weak: Weak<AppWindow>,
) {
    let res = match conv {
        QueuedConversion::Standard(in_path) => {
            let filename = in_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let weak2 = weak.clone();
            let update_progress = move |percentage| {
                let status = slint::format!("↑  Converting  {filename}  {percentage}%");

                let _ = weak2.upgrade_in_event_loop(move |app| {
                    app.global::<UiState<'_>>().set_status(status);
                });
            };

            twbm_core::convert::perform(in_path, &config, &drive_info, &update_progress)
        }
        QueuedConversion::Archive(in_path, out_path) => {
            let filename = out_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let weak2 = weak.clone();
            let update_progress = move |percentage| {
                let status = slint::format!("↓  Archiving  {filename}  {percentage}%");

                let _ = weak2.upgrade_in_event_loop(move |app| {
                    app.global::<UiState<'_>>().set_status(status);
                });
            };

            twbm_core::archive::perform(&in_path, &out_path, &update_progress)
        }
        QueuedConversion::Scrub(game) => {
            let weak2 = weak.clone();
            let game_title = game.title.clone();
            let update_progress = move |percentage| {
                let status = slint::format!("↔  Scrubbing  {game_title}  {percentage}%");

                let _ = weak2.upgrade_in_event_loop(move |app| {
                    app.global::<UiState<'_>>().set_status(status);
                });
            };

            twbm_core::scrub::perform(&game, &config, &drive_info, &update_progress)
        }
    };

    let _ = weak.upgrade_in_event_loop(move |app| {
        let dispatcher = app.global::<Dispatcher<'_>>();

        dispatcher.invoke_dispatch(Message::SetStatus, SharedString::new());

        if let Err(e) = res {
            let text = slint::format!("Conversion failed: {e}");
            dispatcher.invoke_dispatch(Message::NotifyError, text);
        } else {
            dispatcher.invoke_dispatch(Message::TriggerConversion, SharedString::new());
        }

        dispatcher.invoke_dispatch(Message::RefreshAll, SharedString::new());
    });
}
