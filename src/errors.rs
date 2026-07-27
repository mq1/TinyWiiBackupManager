// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(std::io::ErrorKind),

    #[error("JSON error: {0:?} at {1}:{2}")]
    Json(serde_json::error::Category, usize, usize),

    #[error("HTTP error: {0}")]
    Http(#[from] isahc::Error),

    #[error(transparent)]
    WiiDiscInfo(#[from] wii_disc_info::errors::Error),

    #[error("Disc not found")]
    DiscNotFound,

    #[error("Invalid directory name")]
    InvalidDirName,

    #[error("Path is not a directory")]
    NotADir,

    #[error("Hidden directory")]
    HiddenDir,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.kind())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        let category = err.classify();
        let line = err.line();
        let column = err.column();

        Self::Json(category, line, column)
    }
}
