// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, Notification, QueuedConversion, State, game_list};
use slint::{Global, Model, VecModel};

macro_rules! connect {
    ($self:expr, $register:ident($($arg:ident),*) => $handler:ident) => {{
        let weak = $self.as_weak();
        $self.$register(move |$($arg),*| weak.upgrade().unwrap().$handler($($arg),*));
    }};
}

impl State<'_> {
    pub fn handle_callbacks(&self) {
        connect!(self, on_close_notification(i) => close_notification);
        connect!(self, on_apply_sorting() => apply_sorting);
        connect!(self, on_add_to_queue() => add_to_queue);
        connect!(self, on_start_conversion() => start_conversion);
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn close_notification(&self, i: i32) {
        self.get_notifications()
            .as_any()
            .downcast_ref::<VecModel<Notification>>()
            .unwrap()
            .remove(i as usize);
    }

    pub fn apply_sorting(&self) {
        let games = self.get_game_list().games;
        let games = games.as_any().downcast_ref::<VecModel<Game>>().unwrap();

        let sort_by = self.get_config().contents.sort_by;
        let mut sorted = games.iter().collect::<Vec<_>>();
        game_list::sort(&mut sorted, &sort_by);

        games.set_vec(sorted);
    }

    pub fn add_to_queue(&self) {
        let queue = self.get_conversion_queue();
        let queue = queue
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        let adding = self.get_adding_games();
        let adding = adding
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        queue.extend(adding.iter());
        adding.clear();
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
