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

    #[error("Invalid filename")]
    InvalidFilename,

    #[error("Path is not a directory")]
    NotADir,

    #[error("Hidden directory")]
    HiddenDir,

    #[error("Invalid homebrew app meta")]
    InvalidHomebrewAppMeta,

    #[error("Zip error: {0}")]
    Zip(String),

    #[error("Nod disc format error: {0}")]
    NodDiscFormat(String),

    #[error("Nod io error: {0} - {1}")]
    NodIo(String, std::io::ErrorKind),

    #[error("Nod error: {0}")]
    NodOther(String),

    #[error("Hash mismatch for {0}")]
    HashMismatch(String),

    #[error("Failed to get inner writer")]
    IntoInnerWriter,
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

impl From<nod::Error> for Error {
    fn from(err: nod::Error) -> Self {
        match err {
            nod::Error::DiscFormat(err) => Self::NodDiscFormat(err),
            nod::Error::Io(err, io_err) => Self::NodIo(err, io_err.kind()),
            nod::Error::Other(err) => Self::NodOther(err),
        }
    }
}

impl<T> From<std::io::IntoInnerError<T>> for Error {
    fn from(_err: std::io::IntoInnerError<T>) -> Self {
        Self::IntoInnerWriter
    }
}
