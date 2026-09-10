// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{errors::Error, notifications::notification::Notification};

#[derive(Debug, Clone, Default)]
pub struct NotificationList {
    inner: Vec<Notification>,
}

impl NotificationList {
    pub fn info(&mut self, message: impl ToString) {
        self.inner.push(Notification::info(message));
    }

    pub fn error(&mut self, message: impl Into<Error>) {
        self.inner.push(Notification::error(message));
    }

    pub fn warning(&mut self, message: impl ToString) {
        self.inner.push(Notification::warning(message));
    }

    pub fn success(&mut self, message: impl ToString) {
        self.inner.push(Notification::success(message));
    }

    pub fn close(&mut self, index: usize) {
        self.inner.remove(index);
    }

    pub fn has_notifications(&self) -> bool {
        !self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.inner.iter()
    }
}
