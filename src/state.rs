// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, SortBy, State, slint_ext::MyModelExt};
use slint::Global;

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

            let sort_by = state.get_config().contents.sort_by;
            let compare: fn(&Game, &Game) -> std::cmp::Ordering = match sort_by {
                SortBy::NameAscending => |a, b| a.title.cmp(&b.title),
                SortBy::NameDescending => |a, b| b.title.cmp(&a.title),
                SortBy::SizeAscending => |a, b| a.size_gib.total_cmp(&b.size_gib),
                SortBy::SizeDescending => |a, b| b.size_gib.total_cmp(&a.size_gib),
            };

            state.get_game_list().games.sort_by(compare);
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
