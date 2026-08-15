// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug, Clone)]
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

    #[error("Invalid homebrew app meta")]
    InvalidHomebrewAppMeta,

    #[error("Zip error: {0}")]
    Zip(String),
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

impl From<async_zip::error::ZipError> for Error {
    fn from(err: async_zip::error::ZipError) -> Self {
        Self::Zip(err.to_string())
    }
}
