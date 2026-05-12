// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Action, ConversionKind, Logic, QueuedConversion};
use slint::{SharedString, Weak};
use std::path::PathBuf;
use twbm_core::{config::Config, drive_info::DriveInfo};

pub enum Conversion {
    Standard(PathBuf),
    Archive(PathBuf, PathBuf),
    Scrub(PathBuf, SharedString, SharedString),
}

impl Conversion {
    pub fn new(queued: &QueuedConversion) -> Self {
        match queued.kind {
            ConversionKind::Standard => {
                let in_path = PathBuf::from(&queued.in_path);
                Conversion::Standard(in_path)
            }
            ConversionKind::Archive => {
                let in_path = PathBuf::from(&queued.in_path);
                let out_path = PathBuf::from(&queued.out_path);
                Conversion::Archive(in_path, out_path)
            }
            ConversionKind::Scrub => {
                let in_path = PathBuf::from(&queued.in_path);
                let game_title = queued.game_title.clone();
                let game_id = queued.game_id.clone();
                Conversion::Scrub(in_path, game_title, game_id)
            }
        }
    }

    pub fn perform(self, config: Config, drive_info: DriveInfo, weak: Weak<Logic<'static>>) {
        let res = match self {
            Conversion::Standard(in_path) => {
                let filename = in_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = slint::format!("↑  Converting  {filename}  {percentage}%");

                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status);
                    });
                };

                twbm_core::convert::perform(in_path, &config, &drive_info, &update_progress)
            }
            Conversion::Archive(in_path, out_path) => {
                let filename = out_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = slint::format!("↓  Archiving  {filename}  {percentage}%");

                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status);
                    });
                };

                twbm_core::archive::perform(&in_path, &out_path, &update_progress)
            }
            Conversion::Scrub(in_path, game_title, game_id) => {
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = slint::format!("↔  Scrubbing  {game_title}  {percentage}%");

                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status);
                    });
                };

                twbm_core::scrub::perform(
                    &in_path,
                    &game_id,
                    &config,
                    &drive_info,
                    &update_progress,
                )
            }
        };

        let _ = weak.upgrade_in_event_loop(move |logic| {
            logic.invoke_dispatch(Action::SetStatus, SharedString::new());

            if let Err(e) = res {
                let msg = slint::format!("Conversion failed: {e}");
                logic.invoke_dispatch(Action::NotifyError, msg);
            } else {
                logic.invoke_dispatch(Action::TriggerConversion, SharedString::new());
            }

            logic.invoke_dispatch(Action::RefreshAll, SharedString::new());
        });
    }
}
