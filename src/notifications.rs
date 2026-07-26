// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
    pub label: String,
    #[serde(default)]
    pub level: NotificationLevel,
}

impl Notification {
    pub fn new(label: impl ToString, level: NotificationLevel) -> Self {
        Self {
            label: label.to_string(),
            level,
        }
    }

    pub fn info(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Info,
        }
    }

    pub fn warning(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Warning,
        }
    }

    pub fn error(err: impl Into<Error>) -> Self {
        Self {
            label: err.into().to_string(),
            level: NotificationLevel::Error,
        }
    }

    pub fn success(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
            level: NotificationLevel::Success,
        }
    }
}
