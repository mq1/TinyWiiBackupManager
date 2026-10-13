// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        calc_sha1::{HashOutcome, HashResult, HashSummary},
        game::Game,
    },
    message::Message,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Alignment, Element, Font, Length,
    widget::{Column, checkbox, column, row, rule, scrollable, space, text},
};
use lucide_icons::Icon;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct HashSelection {
    games: Vec<(Game, bool)>,
    pub source: PathBuf,
    pub run_id: u64,
    pub running: bool,
    pub results: Vec<HashResult>,
    pub summary: Option<HashSummary>,
    pub progress: String,
}

impl HashSelection {
    pub fn new(games: Vec<Game>, source: PathBuf) -> Self {
        Self {
            games: games.into_iter().map(|game| (game, false)).collect(),
            source,
            run_id: 0,
            running: false,
            results: Vec::new(),
            summary: None,
            progress: String::new(),
        }
    }

    pub fn selected_count(&self) -> usize {
        self.games.iter().filter(|(_, checked)| *checked).count()
    }

    pub fn selected_games(&self) -> Vec<Game> {
        self.games
            .iter()
            .filter(|(_, checked)| *checked)
            .map(|(game, _)| game.clone())
            .collect()
    }

    pub fn set_selected(&mut self, index: usize, selected: bool) {
        if let Some((_, checked)) = self.games.get_mut(index) {
            *checked = selected;
        }
    }

    pub fn select_all(&mut self, selected: bool) {
        for (_, checked) in &mut self.games {
            *checked = selected;
        }
    }

    pub fn start(&mut self, run_id: u64) {
        self.run_id = run_id;
        self.running = true;
        self.results.clear();
        self.summary = None;
        self.progress = format!("Checking {} selected games", self.selected_count());
    }
}

fn result_row(result: &HashResult) -> Element<'_, Message> {
    let disc = result
        .disc
        .map(|disc| format!("Disc {disc}: "))
        .unwrap_or_default();
    let label = match &result.outcome {
        HashOutcome::Known => format!("{disc}SHA1 matches Redump"),
        HashOutcome::Unknown => format!("{disc}SHA1 not found in the bundled Redump database"),
        HashOutcome::Error(error) => format!("{disc}Could not check: {error}"),
    };
    column![
        text(label).size(13),
        result
            .sha1
            .as_ref()
            .map(|sha1| text(sha1).font(Font::MONOSPACE).size(12)),
    ]
    .spacing(4)
    .padding(iced::padding::left(24))
    .into()
}

pub fn check_games(
    selection: &HashSelection,
    busy: bool,
    can_start_new: bool,
) -> Element<'_, Message> {
    let selected = selection.selected_count();
    let all_selected = selected > 0 && selected == selection.games.len();
    let select_all = checkbox(all_selected)
        .label("Select all")
        .on_toggle_maybe((!busy).then_some(Message::SelectAllHashGames));

    let games = selection
        .games
        .iter()
        .enumerate()
        .map(|(index, (game, checked))| {
            let header = row![
                checkbox(*checked)
                    .label(format!("{} [{}]", game.title(), game.id()))
                    .width(Length::Fill)
                    .on_toggle_maybe(
                        (!busy).then_some(move |checked| Message::ToggleHashGame(index, checked))
                    ),
                text(if game.is_wii() { "Wii" } else { "GameCube" }).width(75),
            ]
            .spacing(10)
            .align_y(Alignment::Center);
            let mut item = column![header].spacing(8);
            for result in selection
                .results
                .iter()
                .filter(|result| result.game == *game)
            {
                item = item.push(result_row(result));
            }
            item.into()
        })
        .collect::<Column<'_, Message>>()
        .spacing(16)
        .padding(15);

    let mut check = my_button()
        .label(if selection.running {
            "Checking..."
        } else {
            "Check selected"
        })
        .icon(Icon::SearchCheck)
        .kind(MyButtonKind::Primary);
    if selected > 0 && !busy {
        check = check.on_press(Message::CheckSelectedGames);
    }
    let mut new_check = my_button().label("New check");
    if can_start_new && !busy {
        new_check = new_check.on_press(Message::NewHashSelection);
    }

    my_card(column![
        column![
            text("Check SHA1").size(22),
            text!("Drive: {}", selection.source.display()).size(12),
            row![select_all, space::horizontal(), text!("{} of {} selected", selected, selection.games.len())]
                .align_y(Alignment::Center),
            selection.summary.as_ref().map(|summary| text(summary.to_string()).size(13)),
        ].spacing(10).padding(20),
        rule::horizontal(1),
        scrollable(games).height(Length::Fill),
        rule::horizontal(1),
        column![
            selection.running.then(|| text(&selection.progress).size(13)),
            text(if selection.running {
                "Checks continue if you close this window. Reopen Check SHA1 to see results."
            } else {
                "Compare disc hashes with Redump. An unknown hash does not necessarily mean corruption."
            }).size(12),
            row![new_check, space::horizontal(), my_button().label("Close").on_press(Message::CloseModal), check]
                .spacing(10),
        ].spacing(10).padding(20),
    ].height(520)).width(680).padding(0).into()
}
