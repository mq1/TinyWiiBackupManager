// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use strum_macros::Display;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct Notification {
    label: Box<str>,
    level: NotificationLevel,
    created: Option<OffsetDateTime>,
}

impl Notification {
    pub fn info(label: impl ToString) -> Self {
        Self {
            label: label.to_string().into_boxed_str(),
            level: NotificationLevel::Info,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn warning(label: impl ToString) -> Self {
        Self {
            label: label.to_string().into_boxed_str(),
            level: NotificationLevel::Warning,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn error(label: impl ToString) -> Self {
        Self {
            label: label.to_string().into_boxed_str(),
            level: NotificationLevel::Error,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn success(label: impl ToString) -> Self {
        Self {
            label: label.to_string().into_boxed_str(),
            level: NotificationLevel::Success,
            created: OffsetDateTime::now_local().ok(),
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
        if let Some(created) = self.created {
            write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02} > ",
                created.year(),
                u8::from(created.month()),
                created.day(),
                created.hour(),
                created.minute(),
                created.second()
            )?;
        }

        write!(f, "[{}] {}", self.level, self.label)
    }
}

impl<S: ToString, E: ToString> From<std::result::Result<S, E>> for Notification {
    fn from(result: anyhow::Result<S, E>) -> Self {
        match result {
            Ok(label) => Notification::success(label),
            Err(e) => Notification::error(e),
        }
    }
}

impl From<anyhow::Error> for Notification {
    fn from(e: anyhow::Error) -> Self {
        Notification::error(e)
    }
}
