// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{
    Element, Length,
    widget::{Row, scrollable},
};
use itertools::Itertools;

pub fn view(state: &State) -> Element<'_, Message> {
    let mut row = Row::new().width(Length::Fill).spacing(10).padding(10);

    if !state.hbc_filter.is_empty() {
        let matcher = SkimMatcherV2::default();
        let apps = state
            .hbc_apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                matcher
                    .fuzzy_match(&app.meta.name, &state.hbc_filter)
                    .map(|score| (i, score))
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

    scrollable(row.wrap()).into()
}
