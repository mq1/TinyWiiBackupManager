// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    homebrew::homebrew_state::HomebrewAppList,
    long_operation::LongOperations,
    message::Message,
    notifications::notification_list::NotificationList,
    osc::osc_state::OscAppList,
    ui::{modals::Modal, pages::Page, theme},
};
use iced::{
    Subscription, Task, Theme,
    time::{self, milliseconds},
};
use semver::Version;
use std::path::Path;
use twbm_common::config_contents::ThemePreference;
use twbm_driveinfo::DriveInfoState;
use twbm_games::{import_entry::ImportEntry, manager::GameManager};
use wii_disc_info::game_id::GameID;

#[derive(Debug, Clone)]
pub enum LazyState<T> {
    NotLoaded,
    Loading,
    Loaded(T),
    Errored(String),
}

pub struct AppState {
    pub config: Config,
    pub data_dir: &'static Path,
    pub notifications: NotificationList,
    pub drive: DriveInfoState,
    pub game_manager: GameManager,
    pub homebrew: LazyState<HomebrewAppList>,
    pub current_page: Page,
    pub current_modal: Option<Modal>,
    pub long_operations: LongOperations,
    pub import_queue: Vec<ImportEntry>,
    pub osc_contents: LazyState<OscAppList>,
    pub animation_state: bool,
    pub new_version: Option<Version>,
}

impl AppState {
    pub fn boot(data_dir: &'static Path) -> impl Fn() -> (Self, Task<Message>) {
        move || {
            let state = Self {
                data_dir,
                config: Config::default(),
                notifications: NotificationList::new(data_dir),
                drive: DriveInfoState::Loading,
                game_manager: GameManager::Loading,
                homebrew: LazyState::NotLoaded,
                current_page: Page::Games,
                current_modal: None,
                import_queue: Vec::new(),
                osc_contents: LazyState::NotLoaded,
                animation_state: false,
                new_version: None,
                long_operations: LongOperations::new(),
            };

            let task = state.init_task();

            (state, task)
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::event::listen_with(|event, _status, _id| {
                if let iced::Event::Window(iced::window::Event::FileDropped(path)) = event {
                    Some(Message::FileDropped(path))
                } else {
                    None
                }
            }),
            time::every(milliseconds(500)).map(|_| Message::ToggleAnimationState),
        ])
    }

    pub fn theme(&self) -> Option<Theme> {
        match self.config.contents.theme_preference {
            ThemePreference::System => theme::system(),
            ThemePreference::Light => Some(theme::light()),
            ThemePreference::Dark => Some(theme::dark()),
        }
    }

    pub fn reload_cover(&mut self, game_id: GameID) {
        if let GameManager::Loaded(games) = &mut self.game_manager {
            games.reload_cover(game_id, self.data_dir);
        }
    }

    pub fn reload_all_covers(&mut self) {
        if let GameManager::Loaded(games) = &mut self.game_manager {
            games.reload_all_covers(self.data_dir);
        }
    }

    pub fn reload_osc_icon(&mut self, idx: usize) {
        if let LazyState::Loaded(apps) = &mut self.osc_contents {
            apps.reload_icon(idx, self.data_dir);
        }
    }

    pub fn reload_all_osc_icons(&mut self) {
        if let LazyState::Loaded(apps) = &mut self.osc_contents {
            apps.reload_all_icons(self.data_dir);
        }
    }

    pub fn set_homebrew_apps(&mut self, homebrew: LazyState<HomebrewAppList>) {
        self.homebrew = homebrew;

        if let LazyState::Loaded(apps) = &mut self.homebrew
            && let LazyState::Loaded(osc_apps) = &self.osc_contents
        {
            apps.set_osc_apps(osc_apps);
        }
    }

    pub fn set_osc_contents(&mut self, contents: LazyState<OscAppList>) {
        self.osc_contents = contents;

        if let LazyState::Loaded(osc_apps) = &self.osc_contents
            && let LazyState::Loaded(apps) = &mut self.homebrew
        {
            apps.set_osc_apps(osc_apps);
        }
    }

    pub fn search_games(&mut self, search_term: &str) {
        if let GameManager::Loaded(games) = &mut self.game_manager {
            games.set_search_term(search_term.to_lowercase());
        }
    }

    pub fn search_homebrew_apps(&mut self, search_term: &str) {
        if let LazyState::Loaded(apps) = &mut self.homebrew {
            apps.set_search_term(search_term.to_lowercase());
        }
    }

    pub fn search_osc_apps(&mut self, search_term: &str) {
        if let LazyState::Loaded(apps) = &mut self.osc_contents {
            apps.set_search_term(search_term.to_lowercase());
        }
    }

    pub fn set_disc_info(&mut self, meta: wii_disc_info::Meta) {
        if let Some(Modal::GameInfo((_, current_meta, _))) = &mut self.current_modal {
            *current_meta = Some(meta);
        }
    }

    pub fn set_scrubbable(&mut self, has_update_partition: bool) {
        if let Some(Modal::GameInfo((_, _, scrubbable))) = &mut self.current_modal {
            *scrubbable = has_update_partition;
        }
    }

    pub fn set_show_wii(&mut self, show_wii: bool) {
        if let GameManager::Loaded(games) = &mut self.game_manager {
            games.set_show_wii(show_wii)
        }
    }

    pub fn set_show_ngc(&mut self, show_ngc: bool) {
        if let GameManager::Loaded(games) = &mut self.game_manager {
            games.set_show_wii(show_ngc)
        }
    }
}
