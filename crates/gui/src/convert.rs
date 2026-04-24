// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{ConversionKind, Logic, QueuedConversion};
use slint::{Model, SharedString, ToSharedString, VecModel, Weak};
use std::{path::PathBuf, rc::Rc};
use twbm_core::{config::Config, drive_info::DriveInfo, game::Game};

pub enum Conversion {
    Standard(PathBuf),
    Archive(Game, PathBuf),
    Scrub(Game),
}

impl Conversion {
    pub fn new(queued: &QueuedConversion, games: &Rc<VecModel<(usize, Game)>>) -> Self {
        match queued.kind {
            ConversionKind::Standard => {
                let in_path = PathBuf::from(&queued.path);
                Conversion::Standard(in_path)
            }
            ConversionKind::Archive => {
                let (_, game) = games.row_data(queued.game_idx as usize).unwrap();
                let out_path = PathBuf::from(&queued.path);
                Conversion::Archive(game, out_path)
            }
            ConversionKind::Scrub => {
                let (_, game) = games.row_data(queued.game_idx as usize).unwrap();
                Conversion::Scrub(game)
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
                    let status = format!("Converting  {filename} {percentage}%",);
                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status.to_shared_string());
                    });
                };

                twbm_core::convert::perform(in_path, config, drive_info, &update_progress)
            }
            Conversion::Archive(game, out_path) => {
                let filename = out_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = format!("Archiving {filename}  {percentage}%",);
                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status.to_shared_string());
                    });
                };

                twbm_core::archive::perform(game, out_path, &update_progress)
            }
            Conversion::Scrub(game) => {
                let game_title = game.title.clone();
                let weak2 = weak.clone();
                let update_progress = move |percentage| {
                    let status = format!("Scrubbing {game_title}  {percentage}%");
                    let _ = weak2.upgrade_in_event_loop(move |logic| {
                        logic.set_status(status.to_shared_string());
                    });
                };

                twbm_core::scrub::perform(game, config, drive_info, &update_progress)
            }
        };

        let _ = weak.upgrade_in_event_loop(move |logic| {
            logic.invoke_set_status(SharedString::new());

            if let Err(e) = res {
                logic.invoke_notify_error(e.to_shared_string());
            } else {
                logic.invoke_trigger_conversion();
            }

            logic.invoke_refresh_all();
        });
    }
}
