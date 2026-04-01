// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, Notification, QueuedConversion, State, game_list};
use slint::{Global, Model, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_close_notification(move |i| {
            let state = weak.upgrade().unwrap();

            #[allow(clippy::cast_sign_loss)]
            state
                .get_notifications()
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .remove(i as usize);
        });

        let weak = self.as_weak();
        self.on_apply_sorting(move || {
            let state = weak.upgrade().unwrap();
            let sort_by = state.get_config().contents.sort_by;
            let mut games = state.get_game_list().games.iter().collect::<Vec<_>>();
            game_list::sort(&mut games, &sort_by);

            state
                .get_game_list()
                .games
                .as_any()
                .downcast_ref::<VecModel<Game>>()
                .unwrap()
                .set_vec(games);
        });

        let weak = self.as_weak();
        self.on_add_to_queue(move || {
            let state = weak.upgrade().unwrap();

            let queue = state.get_conversion_queue();
            let queue = queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            let new = state.get_adding_games();
            let new = new
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            queue.extend(new.iter());
            new.clear();
        });

        let weak = self.as_weak();
        self.on_start_conversion(move || weak.upgrade().unwrap().start_conversion());
    }

    pub fn start_conversion(&self) {
        if self.get_is_converting() {
            return;
        }

        let queue = self.get_conversion_queue();
        let queue = queue
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        if queue.row_count() > 0 {
            let conv = queue.remove(0);
            conv.run(self.as_weak());
        }
    }
}
