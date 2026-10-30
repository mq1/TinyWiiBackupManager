// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

//! What the tasks in the task manager do. A job is kept until its task leaves the history, so a
//! failed or cancelled task can be retried.

use crate::{
    config::Config,
    games::{ImportEntry, import::import_game},
    tasks::{Resource, TaskControl, WorkerEvent},
    util::drive_state::DriveState,
};
use smol::stream::Stream;

#[derive(Debug, Clone)]
pub enum Job {
    /// A Wii or GameCube game to the drive. The drive and its settings are those of the moment
    /// the game was added, so choosing another drive meanwhile doesn't change where it goes.
    Import {
        entry: ImportEntry,
        config: Config,
        drive: DriveState,
    },
}

impl Job {
    pub fn title(&self) -> String {
        match self {
            Self::Import { entry, .. } => format!("Import {}", entry.meta().game_title()),
        }
    }

    pub fn resources(&self) -> Vec<Resource> {
        match self {
            Self::Import { config, .. } => {
                vec![Resource::Drive(config.mount_point().to_path_buf())]
            }
        }
    }

    pub fn run(self, control: TaskControl) -> impl Stream<Item = WorkerEvent> {
        match self {
            Self::Import {
                entry,
                config,
                drive,
            } => import_game(entry, config, drive, control),
        }
    }
}
