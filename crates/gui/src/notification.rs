// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Notification;
use slint::ToSharedString;

impl Notification {
    pub fn info(text: impl ToSharedString) -> Notification {
        Notification {
            text: text.to_shared_string(),
            critical: false,
        }
    }

    pub fn error(text: impl ToSharedString) -> Notification {
        Notification {
            text: text.to_shared_string(),
            critical: true,
        }
    }
}

impl From<anyhow::Error> for Notification {
    fn from(value: anyhow::Error) -> Self {
        Notification {
            text: value.to_shared_string(),
            critical: true,
        }
    }
}

impl From<std::io::Error> for Notification {
    fn from(value: std::io::Error) -> Self {
        Notification {
            text: value.to_shared_string(),
            critical: true,
        }
    }
}
