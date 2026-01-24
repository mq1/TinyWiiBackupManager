// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Subscription,
    time::{self, Duration},
};
use std::{fmt::Display, time::Instant};

pub struct Notifications {
    last_id: usize,
    list: Vec<Notification>,
    before: Instant,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            list: Vec::new(),
            before: Instant::now(),
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

    pub fn update(&mut self, now: Instant) {
        let time_passed = now.duration_since(self.before);
        self.before = now;

        for notification in &mut self.list {
            if notification.life != Duration::MAX {
                notification.life = notification.life.saturating_sub(time_passed);
            }
        }

        self.list.retain(|n| !n.life.is_zero());
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
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: text.to_string(),
            level: NotificationLevel::Info,
            life: Duration::from_secs(10),
        });
    }

    pub fn error(&mut self, e: impl AsRef<anyhow::Error>) {
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: e.as_ref().to_string(),
            level: NotificationLevel::Error,
            life: Duration::MAX, // infinite duration
        });
    }

    pub fn success(&mut self, text: impl Display) {
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text: text.to_string(),
            level: NotificationLevel::Success,
            life: Duration::MAX, // infinite duration
        });
    }
}

pub struct Notification {
    pub id: usize,
    pub text: String,
    pub level: NotificationLevel,
    life: Duration,
}

impl Notification {
    pub const fn get_life_str(&self) -> &'static str {
        match self.life.as_millis() / 500 {
            20 => "____________________",
            19 => "___________________",
            18 => "__________________",
            17 => "_________________",
            16 => "________________",
            15 => "_______________",
            14 => "______________",
            13 => "_____________",
            12 => "____________",
            11 => "___________",
            10 => "__________",
            9 => "_________",
            8 => "________",
            7 => "_______",
            6 => "______",
            5 => "_____",
            4 => "____",
            3 => "___",
            2 => "__",
            1 => "_",
            _ => "",
        }
    }
}

pub enum NotificationLevel {
    Info,
    Error,
    Success,
}
