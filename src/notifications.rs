// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Deserialize, Getters, CopyGetters)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    #[getset(get = "pub")]
    label: String,

    #[getset(get_copy = "pub")]
    level: NotificationLevel,
}

impl Notification {
    pub fn new(label: impl Into<String>, level: NotificationLevel) -> Self {
        Self {
            label: label.into(),
            level,
        }
    }
}
