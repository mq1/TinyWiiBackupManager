// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use derive_getters::Getters;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

impl fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationLevel::Info => write!(f, "INFO"),
            NotificationLevel::Warning => write!(f, "WARNING"),
            NotificationLevel::Error => write!(f, "ERROR"),
            NotificationLevel::Success => write!(f, "SUCCESS"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Getters)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
    label: String,
    level: NotificationLevel,
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

    pub fn error(label: impl ToString) -> Self {
        Self {
            label: label.to_string(),
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

#[repr(transparent)]
pub struct Notifications(Vec<Notification>);

impl Notifications {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, notification: Notification) {
        self.0.push(notification);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.0.iter()
    }
}
