// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::futures::TryFutureExt;
use iced::widget::Column;
use iced::{executor, Length};
use iced::{Application, Command, Element, Settings, Theme};
use rfd::{FileDialog, MessageDialog};
use std::sync::Arc;

use crate::pages::Page;
use crate::types::drive::Drive;
use crate::types::game::Game;
use crate::types::message::Message;

mod pages;
mod types;
mod updater;
mod wbfs_file;

pub fn main() -> iced::Result {
    TinyWiiBackupManager::run(Settings::default())
}

pub struct TinyWiiBackupManager {
    page: Page,
    selected_drive: Option<Drive>,
    games: Vec<(Game, bool)>,
    checking_for_updates: bool,
}

impl Application for TinyWiiBackupManager {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self {
                page: Page::Drives,
                selected_drive: None,
                games: vec![],
                checking_for_updates: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("TinyWiiBackupManager")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::SelectDrive(drive) => {
                self.selected_drive = Some(drive);
            }
            Message::OpenDrive => {
                if let Some(drive) = &self.selected_drive {
                    self.page = Page::Games(drive.clone());
                    self.games = drive
                        .get_games()
                        .unwrap()
                        .iter()
                        .map(|g| (g.clone(), false))
                        .collect();
                }
            }
            Message::SelectGame(index, selected) => {
                self.games[index].1 = selected;
            }
            Message::AddGames(drive) => {
                let files = FileDialog::new()
                    .add_filter("WII Game", &["iso", "wbfs"])
                    .pick_files();

                if let Some(files) = files {
                    self.page = Page::AddingGames(files.len());
                    return self.update(Message::AddingGames((drive, files)));
                }
            }
            Message::AddingGames((drive, mut files)) => {
                if files.is_empty() {
                    return self.update(Message::OpenDrive);
                }

                self.page = Page::AddingGames(files.len());

                return Command::perform(
                    async move {
                        let current_game = files.pop().unwrap();
                        drive.add_game(&current_game).unwrap();

                        (drive, files)
                    },
                    Message::AddingGames,
                );
            }
            Message::RemoveGames => {
                let games = self
                    .games
                    .iter()
                    .filter(|(_, checked)| *checked)
                    .map(|(game, _)| game)
                    .collect::<Vec<_>>();

                for game in games {
                    game.delete().unwrap();
                }

                return self.update(Message::OpenDrive);
            }
            Message::CheckForUpdates => {
                self.checking_for_updates = true;

                return Command::perform(
                    updater::check_for_updates().map_err(Arc::from),
                    Message::CheckedForUpdates,
                );
            }
            Message::CheckedForUpdates(res) => {
                self.checking_for_updates = false;

                if let Err(err) = res {
                    let _ = MessageDialog::new()
                        .set_title("Error")
                        .set_description(format!("{}", err))
                        .show();
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match &self.page {
            Page::Drives => pages::drives::view(self),
            Page::Games(drive) => pages::games::view(self, &drive),
            Page::AddingGames(remaining) => pages::adding_games::view(self, *remaining),
        };

        Column::new()
            .push(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
