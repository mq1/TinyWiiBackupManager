// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use humantime::format_rfc3339_seconds;
use smol_str::{SmolStr, ToSmolStr, format_smolstr};
use std::time::SystemTime;
use strum_macros::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display)]
pub enum NotificationLevel {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct Notification {
    label: SmolStr,
    level: NotificationLevel,
    created: SystemTime,
}

impl Notification {
    pub fn info(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Info,
            created: SystemTime::now(),
        }
    }

    pub fn warning(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Warning,
            created: SystemTime::now(),
        }
    }

    pub fn error(err: impl Into<Error>) -> Self {
        Self {
            label: format_smolstr!("{:#}", err.into()),
            level: NotificationLevel::Error,
            created: SystemTime::now(),
        }
    }

    pub fn success(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Success,
            created: SystemTime::now(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn level(&self) -> NotificationLevel {
        self.level
    }
}

impl std::fmt::Display for Notification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} > [{}] {}",
            format_rfc3339_seconds(self.created),
            self.level,
            self.label
        )
    }
}
