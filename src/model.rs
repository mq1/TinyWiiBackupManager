// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Game, HomebrewApp, OscApp, SortBy, State, game, homebrew_app, osc};
use slint::{FilterModel, ModelRc, SharedString, SortModel, VecModel};
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

type SortedModel<T> = SortModel<Rc<VecModel<T>>, Box<dyn Fn(&T, &T) -> Ordering>>;
type FilteredModel<T> = FilterModel<Rc<SortedModel<T>>, Box<dyn Fn(&T) -> bool>>;

pub struct AppModel {
    sort_by: Rc<RefCell<SortBy>>,

    games: Rc<VecModel<Game>>,
    homebrew_apps: Rc<VecModel<HomebrewApp>>,
    osc_apps: Rc<VecModel<OscApp>>,

    sorted_games: Rc<SortedModel<Game>>,
    sorted_homebrew_apps: Rc<SortedModel<HomebrewApp>>,

    games_filter: Rc<RefCell<SharedString>>,
    homebrew_apps_filter: Rc<RefCell<SharedString>>,
    osc_apps_filter: Rc<RefCell<SharedString>>,

    filtered_games: Rc<FilteredModel<Game>>,
    filtered_homebrew_apps: Rc<FilteredModel<HomebrewApp>>,

    // osc_apps is not sorted, just filtered
    filtered_osc_apps: Rc<FilterModel<Rc<VecModel<OscApp>>, Box<dyn Fn(&OscApp) -> bool>>>,
}

impl AppModel {
    pub fn new(sort_by: SortBy) -> Self {
        let sort_by = Rc::new(RefCell::new(sort_by));

        let games = Rc::new(VecModel::from(Vec::new()));
        let homebrew_apps = Rc::new(VecModel::from(Vec::new()));
        let osc_apps = Rc::new(VecModel::from(Vec::new()));

        let compare_games = game::get_compare_fn(sort_by.clone());
        let compare_homebrew_apps = homebrew_app::get_compare_fn(sort_by.clone());

        let sorted_games = Rc::new(SortModel::new(games.clone(), compare_games));
        let sorted_homebrew_apps =
            Rc::new(SortModel::new(homebrew_apps.clone(), compare_homebrew_apps));

        let games_filter = Rc::new(RefCell::new(SharedString::new()));
        let homebrew_apps_filter = Rc::new(RefCell::new(SharedString::new()));
        let osc_apps_filter = Rc::new(RefCell::new(SharedString::new()));

        let filtered_games = Rc::new(FilterModel::new(
            sorted_games.clone(),
            game::get_filter_fn(games_filter.clone()),
        ));
        let filtered_homebrew_apps = Rc::new(FilterModel::new(
            sorted_homebrew_apps.clone(),
            homebrew_app::get_filter_fn(homebrew_apps_filter.clone()),
        ));
        let filtered_osc_apps = Rc::new(FilterModel::new(
            osc_apps.clone(),
            osc::get_filter_fn(osc_apps_filter.clone()),
        ));

        Self {
            sort_by,
            games,
            homebrew_apps,
            osc_apps,
            sorted_games,
            sorted_homebrew_apps,
            games_filter,
            homebrew_apps_filter,
            osc_apps_filter,
            filtered_games,
            filtered_homebrew_apps,
            filtered_osc_apps,
        }
    }

    pub fn init_state(&self, state: &State<'_>) {
        state.set_games(ModelRc::from(self.filtered_games.clone()));
        state.set_homebrew_apps(ModelRc::from(self.filtered_homebrew_apps.clone()));
        state.set_osc_apps(ModelRc::from(self.filtered_osc_apps.clone()));
    }

    pub fn set_sort_by(&self) -> Box<dyn Fn(SortBy)> {
        let sort_by = self.sort_by.clone();
        let sorted_games = self.sorted_games.clone();
        let sorted_homebrew_apps = self.sorted_homebrew_apps.clone();

        Box::new(move |new| {
            *sort_by.borrow_mut() = new;
            sorted_games.reset();
            sorted_homebrew_apps.reset();
        })
    }

    pub fn set_games_filter(&self) -> Box<dyn Fn(SharedString)> {
        let games_filter = self.games_filter.clone();
        let filtered_games = self.filtered_games.clone();

        Box::new(move |new| {
            *games_filter.borrow_mut() = new;
            filtered_games.reset();
        })
    }

    pub fn set_homebrew_apps_filter(&self) -> Box<dyn Fn(SharedString)> {
        let homebrew_apps_filter = self.homebrew_apps_filter.clone();
        let filtered_homebrew_apps = self.filtered_homebrew_apps.clone();

        Box::new(move |new| {
            *homebrew_apps_filter.borrow_mut() = new;
            filtered_homebrew_apps.reset();
        })
    }

    pub fn set_osc_apps_filter(&self) -> Box<dyn Fn(SharedString)> {
        let osc_apps_filter = self.osc_apps_filter.clone();
        let filtered_osc_apps = self.filtered_osc_apps.clone();

        Box::new(move |new| {
            *osc_apps_filter.borrow_mut() = new;
            filtered_osc_apps.reset();
        })
    }

    pub fn set_games(&self) -> Box<dyn Fn(Vec<Game>)> {
        let games = self.games.clone();

        Box::new(move |new| {
            games.clear();
            games.extend(new);
        })
    }

    pub fn set_homebrew_apps(&self) -> Box<dyn Fn(Vec<HomebrewApp>)> {
        let homebrew_apps = self.homebrew_apps.clone();

        Box::new(move |new| {
            homebrew_apps.clear();
            homebrew_apps.extend(new);
        })
    }

    pub fn set_osc_apps(&self) -> Box<dyn Fn(Vec<OscApp>)> {
        let osc_apps = self.osc_apps.clone();

        Box::new(move |new| {
            osc_apps.clear();
            osc_apps.extend(new);
        })
    }
}
