// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Notification, OscContents, State, checksum, covers};
use slint::{Global, Model, ModelRc, VecModel};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

impl State<'_> {
    pub fn handle_callbacks(&self, data_dir: &'static Path) {
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

        let weak = self.as_weak();
        self.on_cache_covers(move || {
            let state = weak.upgrade().unwrap();
            state.cache_covers();
        });

        let weak = self.as_weak();
        self.on_load_osc_contents(move || {
            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                let raw = OscContents::fetch(&data_dir);
                let _ = weak.upgrade_in_event_loop(move |state| {
                    let res = match raw {
                        Ok(raw) => OscContents::load(raw).into(),
                        Err(e) => Err(e).into(),
                    };

                    state.invoke_got_osc_contents(res);
                });
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

    pub fn cache_covers(&self) {
        let ids = self
            .get_game_list()
            .games
            .iter()
            .map(|g| g.id.to_string())
            .collect::<Vec<_>>();

        let data_dir = PathBuf::from(&self.get_data_dir());

        let weak = self.as_weak();
        let _ = std::thread::spawn(move || {
            for game_id in ids {
                if let Err(e) = covers::cache_cover(&game_id, &data_dir) {
                    eprintln!("Failed to cache cover for {game_id}: {e}");
                }
                let _ = weak.upgrade_in_event_loop(move |state| {
                    state.reload_covers();
                });
            }
        });
    }

    pub fn reload_covers(&self) {
        let mut model = self.get_game_list();
        let mut games = model.games.iter().collect::<Vec<_>>();

        let data_dir = self.get_data_dir();
        let data_dir = Path::new(&data_dir);

        for game in &mut games {
            game.reload_cover(data_dir);
        }

        model.games = ModelRc::from(Rc::new(VecModel::from(games)));
        self.set_game_list(model);
    }
}
