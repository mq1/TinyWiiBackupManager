// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, Notification, QueuedConversion, State, game_list};
use slint::{Global, Model, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_close_notification(move |i| {
            let state = weak.upgrade().unwrap();

            let notifications = state.get_notifications();
            let notifications = notifications
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap();

            #[allow(clippy::cast_sign_loss)]
            notifications.remove(i as usize);
        });

        let weak = self.as_weak();
        self.on_apply_sorting(move || {
            let state = weak.upgrade().unwrap();

            let games = state.get_game_list().games;
            let games = games.as_any().downcast_ref::<VecModel<Game>>().unwrap();

            let sort_by = state.get_config().contents.sort_by;
            let mut sorted = games.iter().collect::<Vec<_>>();
            game_list::sort(&mut sorted, &sort_by);

            games.set_vec(sorted);
        });

        let weak = self.as_weak();
        self.on_add_to_queue(move || {
            let state = weak.upgrade().unwrap();

            let queue = state.get_conversion_queue();
            let queue = queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            let adding = state.get_adding_games();
            let adding = adding
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            queue.extend(adding.iter());
            adding.clear();
        });

        let weak = self.as_weak();
        self.on_start_conversion(move || {
            let state = weak.upgrade().unwrap();

            if state.get_is_converting() {
                return;
            }

            let queue = state.get_conversion_queue();
            let queue = queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            if queue.row_count() > 0 {
                let conv = queue.remove(0);
                conv.run(state.as_weak());
            }
        });
    }
}
