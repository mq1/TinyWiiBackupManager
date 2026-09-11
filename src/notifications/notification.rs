// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use derive_getters::Getters;
use strum_macros::Display;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display)]
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

    created: Option<OffsetDateTime>,
}

impl Notification {
    pub fn info(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Info,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn warning(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Warning,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn error(err: impl Into<Error>) -> Self {
        Self {
            label: format!("{:#}", err.into()),
            level: NotificationLevel::Error,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn success(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Success,
            created: OffsetDateTime::now_local().ok(),
        }
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(created) = &self.created {
            write!(f, "{created} > ")?;
        }

        write!(f, "[{}] {}", self.level, self.label)
    }
}
