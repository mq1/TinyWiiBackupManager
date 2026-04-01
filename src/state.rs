// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Notification, State};
use slint::{Model, SharedString, VecModel};

impl State<'_> {
    fn notify(&self, new: Notification) {
        let notifications = self.get_notifications();
        let array = notifications
            .as_any()
            .downcast_ref::<VecModel<Notification>>()
            .unwrap();

        array.push(new);
    }

    pub fn notify_info(&self, text: impl Into<SharedString>) {
        let new = Notification {
            text: text.into(),
            critical: false,
        };

        self.notify(new);
    }

    pub fn notify_err(&self, text: impl Into<SharedString>) {
        let new = Notification {
            text: text.into(),
            critical: true,
        };

        self.notify(new);
    }

    pub fn close_notification(&self, i: i32) {
        #[allow(clippy::cast_sign_loss)]
        self.get_notifications()
            .as_any()
            .downcast_ref::<VecModel<Notification>>()
            .unwrap()
            .remove(i as usize);
    }
}
