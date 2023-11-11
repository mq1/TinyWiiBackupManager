// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::types::drive::Drive;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    SelectDrive(Drive),
    OpenDrive,
    SelectGame(usize, bool),
    AddGames(Drive),
    AddingGames((Drive, Vec<PathBuf>)),
    RemoveGames,
    CheckForUpdates,
}
