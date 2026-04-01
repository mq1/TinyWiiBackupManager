// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Notification, State};
use slint::{Global, Model, VecModel};

pub fn handle_callbacks(state: &State<'_>) {
    let weak = state.as_weak();
    state.on_close_notification(move |i| {
        let state = weak.upgrade().unwrap();

        #[allow(clippy::cast_sign_loss)]
        state
            .get_notifications()
            .as_any()
            .downcast_ref::<VecModel<Notification>>()
            .unwrap()
            .remove(i as usize);
    });
}
