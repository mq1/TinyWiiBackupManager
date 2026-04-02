// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::slint_ext::MyModelExt;
use crate::{Game, State, game_list};
use slint::{Global, Model, VecModel};

impl State<'_> {
    pub fn handle_callbacks(&self) {
        let weak = self.as_weak();
        self.on_close_notification(move |i| {
            let state = weak.upgrade().unwrap();
            state.get_notifications().remove(i);
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

            state
                .get_conversion_queue()
                .append(state.get_adding_games());
        });

        let weak = self.as_weak();
        self.on_start_conversion(move || {
            let state = weak.upgrade().unwrap();

            if state.get_is_converting() {
                return;
            }

            if let Some(conv) = state.get_conversion_queue().pop_first() {
                conv.run(weak.clone());
            }
        });
    }
}
