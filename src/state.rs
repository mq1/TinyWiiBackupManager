// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use crate::{Notification, QueuedConversion, State, checksum};
use slint::{Global, Model, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_start_conversion(move || {
            let state = weak.upgrade().unwrap();
            state.start_conversion();
        });

        let weak = self.as_weak();
        self.on_checksum(move |game| {
            let weak = weak.clone();
            let game_dir = PathBuf::from(&game.path);
            let is_wii = game.is_wii;
            let game_id = game.id.to_string();
            let _ = std::thread::spawn(move || {
                if let Err(e) = checksum::perform(game_dir, is_wii, &game_id, &weak) {
                    let _ = weak.upgrade_in_event_loop(move |state| {
                        state.push_notification(e.into());
                    });
                }
            });
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
