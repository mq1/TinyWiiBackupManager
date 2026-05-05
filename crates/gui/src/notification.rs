// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Notification;
use slint::SharedString;

impl Notification {
    pub fn info(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            critical: false,
        }
    }

    pub fn error(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            critical: true,
        }
    }
}
