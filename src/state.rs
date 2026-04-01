// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, Notification, State, game_list};
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
    }
}
