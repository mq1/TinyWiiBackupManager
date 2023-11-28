// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use crate::types::drive::Drive;
use crate::types::game::Game;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Message {
    SelectDrive(Drive),
    OpenDrive,
    GotGames(Result<Vec<Game>, Arc<anyhow::Error>>),
    SelectGame(usize, bool),
    AddGames(Drive),
    AddingGames((Drive, Vec<PathBuf>)),
    RemoveGames,
    CheckForUpdates,
    CheckedForUpdates(Result<(), Arc<anyhow::Error>>),
    Error(Arc<anyhow::Error>),
}
