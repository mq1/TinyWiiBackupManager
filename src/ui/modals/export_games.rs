// SPDX-FileCopyrightText: 2026 TinyWiiBackupManager contributors
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        export::{DEFAULT_PARALLEL_EXPORTS, ExportFormat},
        game::Game,
    },
    message::Message,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{
    Alignment, Element, Length,
    widget::{Column, checkbox, column, pick_list, row, rule, scrollable, space, text},
};
use lucide_icons::Icon;

#[derive(Debug, Clone)]
pub struct ExportSelection {
    games: Vec<(Game, bool)>,
    pub format: ExportFormat,
    pub parallel_jobs: usize,
}

impl ExportSelection {
    pub fn new(games: Vec<Game>) -> Self {
        Self {
            games: games.into_iter().map(|game| (game, false)).collect(),
            format: ExportFormat::default(),
            parallel_jobs: DEFAULT_PARALLEL_EXPORTS,
        }
    }

    pub fn selected_count(&self) -> usize {
        self.games.iter().filter(|(_, selected)| *selected).count()
    }

    pub fn selected_games(&self) -> Vec<Game> {
        self.games
            .iter()
            .filter(|(_, selected)| *selected)
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
}

pub fn export_games<'a>(selection: &'a ExportSelection, state: &AppState) -> Element<'a, Message> {
    let picking = state.export_picker.is_some();
    let selected_count = selection.selected_count();
    let all_selected = selected_count > 0 && selected_count == selection.games.len();
    let select_all = checkbox(all_selected)
        .label("Select all")
        .on_toggle_maybe((!picking).then_some(Message::SelectAllExportGames));

    let games =
        selection
            .games
            .iter()
            .enumerate()
            .map(|(index, (game, selected))| {
                row![
                    checkbox(*selected)
                        .label(format!("{} [{}]", game.title(), game.id()))
                        .width(Length::Fill)
                        .on_toggle_maybe((!picking).then_some(move |selected| {
                            Message::ToggleExportGame(index, selected)
                        })),
                    text(if game.is_wii() { "Wii" } else { "GameCube" }).width(75),
                    text(game.size_str()).width(80),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into()
            })
            .collect::<Column<'_, Message>>()
            .spacing(12)
            .padding(10);

    let format: Element<'_, Message> = if picking {
        text(selection.format.to_string()).into()
    } else {
        pick_list(
            ExportFormat::ALL,
            Some(selection.format),
            Message::SetExportFormat,
        )
        .into()
    };
    let parallel_jobs: Element<'_, Message> = if picking {
        text(selection.parallel_jobs.to_string()).into()
    } else {
        pick_list(
            [1, 2, 3, 4],
            Some(selection.parallel_jobs),
            Message::SetExportParallelJobs,
        )
        .into()
    };

    let mut export = my_button()
        .label(if picking {
            "Choosing folder..."
        } else {
            "Export selected..."
        })
        .icon(Icon::HardDriveDownload)
        .kind(MyButtonKind::Primary);
    if selected_count > 0 && !picking {
        export = export.on_press(Message::PickBatchExportDestination);
    }

    my_card(
        column![
            column![
                text("Export games").size(22),
                text("Choose games from this drive and export them to one folder."),
                row![
                    select_all,
                    space::horizontal(),
                    text!("{} of {} selected", selected_count, selection.games.len()),
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(12)
            .padding(20),
            rule::horizontal(1),
            scrollable(games).height(Length::Fill),
            rule::horizontal(1),
            column![
                row![text("Format"), format]
                    .spacing(10)
                    .align_y(Alignment::Center),
                row![text("Simultaneous games"), parallel_jobs]
                    .spacing(10)
                    .align_y(Alignment::Center),
                text("More simultaneous games may be slower on a USB drive.").size(12),
                text("Existing files are kept. Any failures are reported after the export.")
                    .size(12),
                row![
                    space::horizontal(),
                    my_button().label("Cancel").on_press(Message::CloseModal),
                    export,
                ]
                .spacing(10),
            ]
            .spacing(12)
            .padding(20),
        ]
        .height(580),
    )
    .padding(0)
    .width(680)
    .into()
}
