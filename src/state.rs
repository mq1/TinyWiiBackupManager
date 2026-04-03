// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, Notification, QueuedConversion, SortBy, State};
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
        self.on_apply_sorting(move || {
            let state = weak.upgrade().unwrap();

            let sort_by = state.get_config().contents.sort_by;
            let compare: fn(&Game, &Game) -> std::cmp::Ordering = match sort_by {
                SortBy::NameAscending => |a, b| a.title.cmp(&b.title),
                SortBy::NameDescending => |a, b| b.title.cmp(&a.title),
                SortBy::SizeAscending => |a, b| a.size_gib.total_cmp(&b.size_gib),
                SortBy::SizeDescending => |a, b| b.size_gib.total_cmp(&a.size_gib),
            };

            let mut games = state.get_game_list().games.iter().collect::<Vec<_>>();
            games.sort_by(compare);

            let model = state.get_game_list().games;
            model
                .as_any()
                .downcast_ref::<VecModel<Game>>()
                .unwrap()
                .set_vec(games);
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

            if state.get_is_converting() {
                return;
            }

            let model = state.get_conversion_queue();
            let model = model
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            if model.row_count() > 0 {
                let conv = model.remove(0);
                conv.run(weak.clone());
            }
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
}
