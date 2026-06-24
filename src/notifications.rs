// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
    pub label: String,
    pub level: NotificationLevel,
}

impl From<String> for Notification {
    fn from(label: String) -> Self {
        Self {
            label,
            level: NotificationLevel::Info,
        }
    }
}

impl From<anyhow::Error> for Notification {
    fn from(e: anyhow::Error) -> Self {
        Self {
            label: e.to_string(),
            level: NotificationLevel::Error,
        }
    }
}

pub struct Notifications(Vec<Notification>);

impl Notifications {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, notification: impl Into<Notification>) {
        let notification = notification.into();
        eprintln!("{}: {}", notification.level, notification.label);
        self.0.push(notification);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.0.iter()
    }
}
