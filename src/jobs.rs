// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

//! What the tasks in the task manager do. A job is kept until its task leaves the history, so a
//! failed or cancelled task can be retried.

use crate::{
    config::Config,
    games::{
        ImportEntry, calc_sha1::calc_sha1, convert::convert_game, export::export_game, game::Game,
        import::import_game, scrub::scrub_game,
    },
    tasks::{Resource, TaskControl, WorkerEvent},
    util::drive_state::DriveState,
};
use smol::stream::{Stream, StreamExt};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub enum Job {
    /// A Wii or GameCube game to the drive. The drive and its settings are those of the moment
    /// the game was added, so choosing another drive meanwhile doesn't change where it goes.
    Import {
        entry: ImportEntry,
        config: Config,
        drive: DriveState,
    },
    /// A disc image into another format, from one file to another (Toolbox).
    Convert {
        disc_path: PathBuf,
        out_path: PathBuf,
    },
    /// A game from the drive at `mount_point` to a file.
    Export {
        game: Game,
        out_path: PathBuf,
        mount_point: PathBuf,
    },
    /// Removes the update partition of a Wii game on the drive at `mount_point`.
    Scrub { game: Game, mount_point: PathBuf },
    /// Checks a game's checksum against the known good dumps; only reads.
    Hash { game: Game, mount_point: PathBuf },
}

/// The folder a file is in, standing for its drive: the task manager doesn't know mount points
/// of arbitrary paths, so two folders on one disk count as two resources.
fn folder_of(path: &Path) -> Resource {
    Resource::Drive(path.parent().unwrap_or(path).to_path_buf())
}

impl Job {
    pub fn title(&self) -> String {
        match self {
            Self::Import { entry, .. } => format!("Import {}", entry.meta().game_title()),
            Self::Convert { out_path, .. } => format!(
                "Convert into {}",
                out_path
                    .file_name()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
            ),
            Self::Export { game, .. } => format!("Export {}", game.title()),
            Self::Scrub { game, .. } => format!("Scrub {}", game.title()),
            Self::Hash { game, .. } => format!("Check the hash of {}", game.title()),
        }
    }

    pub fn resources(&self) -> Vec<Resource> {
        match self {
            Self::Import { config, .. } => {
                vec![Resource::Drive(config.mount_point().to_path_buf())]
            }
            Self::Convert {
                disc_path,
                out_path,
            } => {
                let mut resources = vec![folder_of(disc_path), folder_of(out_path)];
                resources.dedup();
                resources
            }
            Self::Export {
                out_path,
                mount_point,
                ..
            } => vec![Resource::Drive(mount_point.clone()), folder_of(out_path)],
            Self::Scrub { mount_point, .. } | Self::Hash { mount_point, .. } => {
                vec![Resource::Drive(mount_point.clone())]
            }
        }
    }

    /// The drive whose list of games changes when the job is done, so it is reloaded if shown.
    pub fn changes_games_on(&self) -> Option<&Path> {
        match self {
            Self::Import { config, .. } => Some(config.mount_point()),
            Self::Scrub { mount_point, .. } => Some(mount_point),
            Self::Convert { .. } | Self::Export { .. } | Self::Hash { .. } => None,
        }
    }

    /// Whether this job scrubs the game in `game_dir`.
    pub fn scrubs(&self, game_dir: &Path) -> bool {
        matches!(self, Self::Scrub { game, .. } if game.path() == game_dir)
    }

    pub fn run(self, control: TaskControl) -> impl Stream<Item = WorkerEvent> {
        match self {
            Self::Import {
                entry,
                config,
                drive,
            } => import_game(entry, config, drive, control).boxed(),
            Self::Convert {
                disc_path,
                out_path,
            } => convert_game(disc_path, out_path, control).boxed(),
            Self::Export { game, out_path, .. } => export_game(game, out_path, control).boxed(),
            Self::Scrub { game, .. } => scrub_game(game, control).boxed(),
            Self::Hash { game, .. } => calc_sha1(game, control).boxed(),
        }
    }
}
