// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::DisplayedNotification;
use slint::ToSharedString;

impl DisplayedNotification {
    pub fn info(text: impl ToSharedString) -> Self {
        Self {
            text: text.to_shared_string(),
            critical: false,
        }
    }

    pub fn error(text: impl ToSharedString) -> Self {
        Self {
            text: text.to_shared_string(),
            critical: true,
        }
    }
}

impl From<anyhow::Error> for DisplayedNotification {
    fn from(value: anyhow::Error) -> Self {
        Self {
            text: value.to_shared_string(),
            critical: true,
        }
    }
}

impl From<std::io::Error> for DisplayedNotification {
    fn from(value: std::io::Error) -> Self {
        Self {
            text: value.to_shared_string(),
            critical: true,
        }
    }
}
