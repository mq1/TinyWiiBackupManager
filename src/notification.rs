// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Notification;
use slint::{SharedString, ToSharedString};

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

impl From<&str> for Notification {
    fn from(value: &str) -> Self {
        Notification {
            text: value.to_shared_string(),
            critical: false,
        }
    }
}

impl From<String> for Notification {
    fn from(value: String) -> Self {
        Notification {
            text: value.to_shared_string(),
            critical: false,
        }
    }
}

impl From<SharedString> for Notification {
    fn from(value: SharedString) -> Self {
        Notification {
            text: value,
            critical: false,
        }
    }
}
