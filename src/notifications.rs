// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use derive_getters::Getters;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Notifications {
    last_id: usize,
    list: Vec<Notification>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            list: Vec::new(),
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Notification> {
        self.list.iter().rev()
    }

    pub fn close(&mut self, id: usize) {
        // note: we could have binary-searched (as the ids are always ordered)
        // but as we don't have hundreds of notifications this is fine

        self.list
            .iter()
            .position(|n| n.id == id)
            .map(|i| self.list.remove(i));
    }

    pub fn info(&mut self, text: impl Display) {
        println!("INFO: {text}");

        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: text.to_string(),
            level: NotificationLevel::Info,
        });
    }

    pub fn error(&mut self, e: impl AsRef<anyhow::Error>) {
        eprintln!("ERROR: {}", e.as_ref());

        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: e.as_ref().to_string(),
            level: NotificationLevel::Error,
        });
    }

    pub fn success(&mut self, text: impl Display) {
        println!("SUCCESS: {text}");

        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: text.to_string(),
            level: NotificationLevel::Success,
        });
    }
}

#[derive(Debug, Clone, Getters)]
pub struct Notification {
    #[getter(copy)]
    id: usize,
    text: String,
    #[getter(copy)]
    level: NotificationLevel,
}

#[derive(Debug, Copy, Clone)]
pub enum NotificationLevel {
    Info,
    Error,
    Success,
}
