// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use smol_str::{SmolStr, ToSmolStr};
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

#[derive(Debug, Clone)]
pub struct Notification {
    label: SmolStr,
    level: NotificationLevel,
    created: Option<OffsetDateTime>,
}

impl Notification {
    pub fn info(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Info,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn warning(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Warning,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn error(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
            level: NotificationLevel::Error,
            created: OffsetDateTime::now_local().ok(),
        }
    }

    pub fn success(label: impl ToSmolStr) -> Self {
        Self {
            label: label.to_smolstr(),
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
                "{}-{}-{} {}:{}:{} > ",
                created.year(),
                created.month(),
                created.day(),
                created.hour(),
                created.minute(),
                created.second()
            )?;
        }

        write!(f, "[{}] {}", self.level, self.label)
    }
}

impl<S: ToSmolStr, E: ToSmolStr> From<std::result::Result<S, E>> for Notification {
    fn from(result: anyhow::Result<S, E>) -> Self {
        match result {
            Ok(label) => Notification::success(label.to_smolstr()),
            Err(e) => Notification::error(e.to_smolstr()),
        }
    }
}

impl From<anyhow::Error> for Notification {
    fn from(e: anyhow::Error) -> Self {
        Notification::error(e)
    }
}
