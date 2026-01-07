// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{
    Element, Length,
    widget::{Row, column, container, row, scrollable, text},
};
use itertools::Itertools;
use lucide_icons::iced::{icon_arrow_down_left, icon_hard_drive, icon_waves};

pub fn view(state: &State) -> Element<'_, Message> {
    if !state.config.valid_mount_point() {
        return container(
            row![
                icon_arrow_down_left(),
                text("Click on"),
                icon_hard_drive(),
                text("to select a Drive/Mount Point")
            ]
            .spacing(5),
        )
        .center(Length::Fill)
        .into();
    }

    let mut row = Row::new().width(Length::Fill).spacing(10);

    if !state.hbc_filter.is_empty() {
        let matcher = SkimMatcherV2::default();
        let apps = state
            .hbc_apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                let score = matcher.fuzzy_match(&app.meta.name, &state.hbc_filter);
                score.map(|score| (i, score))
            })
            .sorted_unstable_by_key(|(_, score)| *score);

        for (i, _) in apps {
            row = row.push(components::hbc_card::view(state, i));
        }
    } else {
        for i in 0..state.hbc_apps.len() {
            row = row.push(components::hbc_card::view(state, i));
        }
    }

    column![
        components::hbc_toolbar::view(state),
        scrollable(
            column![
                row![
                    icon_waves().size(18),
                    text("Homebrew Channel Apps").size(18)
                ]
                .spacing(5),
                row.wrap()
            ]
            .spacing(10)
            .padding(10)
        ),
    ]
    .into()
}
