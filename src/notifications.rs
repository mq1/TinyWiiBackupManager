// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use getset::{CopyGetters, Getters};

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::EnumString)]
#[strum(ascii_case_insensitive)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Debug, Clone, Getters, CopyGetters)]
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
