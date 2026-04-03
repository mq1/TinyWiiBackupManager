// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Notification, QueuedConversion, State};
use slint::{Global, Model, ModelRc, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_notify(move |notification| {
            let state = weak.upgrade().unwrap();
            state.push_notification(notification);
        });

        let weak = self.as_weak();
        self.on_close_notification(move |i| {
            let state = weak.upgrade().unwrap();

            let model = state.get_notifications();

            #[allow(clippy::cast_sign_loss)]
            model
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .remove(i as usize);
        });

        let weak = self.as_weak();
        self.on_add_to_queue(move || {
            let state = weak.upgrade().unwrap();

            let model = state.get_conversion_queue();
            model
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap()
                .extend(state.get_adding_games().iter());
            state.set_adding_games(ModelRc::default());
        });

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
