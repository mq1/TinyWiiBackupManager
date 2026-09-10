// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use derive_getters::Getters;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Getters)]
pub struct Notification {
    label: String,

    #[getter(copy)]
    level: NotificationLevel,
}

impl Notification {
    pub(super) fn info(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Info,
        }
    }

    pub(super) fn warning(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Warning,
        }
    }

    pub(super) fn error(err: impl Into<Error>) -> Self {
        Self {
            label: format!("{:#}", err.into()),
            level: NotificationLevel::Error,
        }
    }

    pub(super) fn success(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Success,
        }
    }
}
