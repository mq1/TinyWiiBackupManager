// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, HomebrewApp, OscAppMeta, SortBy, State, game, homebrew_app};
use slint::{ModelRc, SortModel, VecModel};
use std::{cmp::Ordering, rc::Rc};

type SortedModel<T> = SortModel<Rc<VecModel<T>>, Box<dyn Fn(&T, &T) -> Ordering>>;

pub struct AppModel {
    pub games: Rc<VecModel<Game>>,
    pub homebrew_apps: Rc<VecModel<HomebrewApp>>,
    pub osc_apps: Rc<VecModel<OscAppMeta>>,

    pub sorted_games: Rc<SortedModel<Game>>,
    pub sorted_homebrew_apps: Rc<SortedModel<HomebrewApp>>,
}

impl AppModel {
    pub fn new(sort_by: SortBy) -> Self {
        let games = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps = Rc::new(VecModel::from(Vec::new()));

        let compare_games = game::get_compare_fn(sort_by);
        let compare_homebrew_apps = homebrew_app::get_compare_fn(sort_by);

        let sorted_games = Rc::new(SortModel::new(games.clone(), compare_games));
        let sorted_homebrew_apps =
            Rc::new(SortModel::new(homebrew_apps.clone(), compare_homebrew_apps));

        Self {
            games,
            homebrew_apps,
            osc_apps,
            sorted_games,
            sorted_homebrew_apps,
        }
    }

    pub fn init_state(&self, state: &State<'_>) {
        state.set_games(ModelRc::from(self.sorted_games.clone()));
        state.set_homebrew_apps(ModelRc::from(self.sorted_homebrew_apps.clone()));
        state.set_osc_apps(ModelRc::from(self.osc_apps.clone()));
    }
}
