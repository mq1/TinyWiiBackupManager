// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{
    Subscription,
    time::{self, Duration},
};

#[inline]
pub fn notifications_subscription(_state: &State) -> Subscription<Message> {
    time::every(Duration::from_millis(500)).map(|_| Message::NotificationTick)
}

pub struct Notifications {
    last_id: usize,
    pub list: Vec<Notification>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            last_id: 0,
            list: Vec::new(),
        }
    }

    /// 1/2 of a second has passed
    pub fn tick(&mut self) {
        for notification in &mut self.list {
            if notification.life != u8::MAX {
                notification.life -= 1;
            }
        }

        self.list.retain(|n| n.life != 0);
    }

    pub fn close(&mut self, id: usize) {
        // note: we could have binary-searched (as the ids are always ordered)
        // but as we don't have hundreds of notifications this is fine

        self.list
            .iter()
            .position(|n| n.id == id)
            .map(|i| self.list.remove(i));
    }

    pub fn info(&mut self, text: String) {
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text,
            level: NotificationLevel::Info,
            life: 20, // 10 seconds
        });
    }

    pub fn error(&mut self, text: String) {
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text,
            level: NotificationLevel::Error,
            life: u8::MAX, // infinite duration
        });
    }

    pub fn success(&mut self, text: String) {
        self.last_id += 1;

        self.list.push(Notification {
            id: self.last_id,
            text,
            level: NotificationLevel::Success,
            life: u8::MAX, // infinite duration
        });
    }
}

pub struct Notification {
    pub id: usize,
    pub text: String,
    pub level: NotificationLevel,
    life: u8,
}

impl Notification {
    pub fn get_life_str(&self) -> &'static str {
        match self.life {
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
