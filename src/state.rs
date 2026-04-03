// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Notification, QueuedConversion, State};
use slint::{Global, Model, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_start_conversion(move || {
            let state = weak.upgrade().unwrap();
            state.start_conversion();
        });
    }

    pub fn push_notification(&self, notification: Notification) {
        let model = self.get_notifications();
        model
            .as_any()
            .downcast_ref::<VecModel<Notification>>()
            .unwrap()
            .push(notification);
    }

    pub fn start_conversion(&self) {
        if self.get_is_converting() {
            return;
        }

        let model = self.get_conversion_queue();
        let model = model
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        if model.row_count() > 0 {
            let conv = model.remove(0);
            conv.run(self.as_weak());
        }
    }
}
