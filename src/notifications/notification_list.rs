// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::notifications::notification::Notification;

#[derive(Debug, Clone, Default)]
pub struct NotificationList {
    inner: Vec<Notification>,
}

impl NotificationList {
    pub fn close(&mut self, index: usize) {
        self.inner.remove(index);
    }

    pub fn add(&mut self, notification: Notification) {
        self.inner.push(notification);
    }

    pub fn has_notifications(&self) -> bool {
        !self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.inner.iter()
    }
}
