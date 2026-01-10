// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{Screen, style},
};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{
    Element, Length,
    widget::{button, container, row, table, text},
};
use itertools::Itertools;
use lucide_icons::iced::{icon_info, icon_trash};

pub fn view(state: &State) -> Element<'_, Message> {
    let t_columns = vec![
        table::column(text("Name").size(16), |i: usize| {
            text(&state.hbc_apps[i].meta.name)
        }),
        table::column(text("Version").size(16), |i: usize| {
            text(&state.hbc_apps[i].meta.version)
        }),
        table::column(text("Size").size(16), |i: usize| {
            text(state.hbc_apps[i].size.to_string())
        }),
        table::column(text("Actions").size(16), |i: usize| {
            row![
                button(row![icon_info(), text("Info")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::NavigateTo(Screen::HbcInfo(i))),
                button(row![icon_trash(), text("Delete")].spacing(5))
                    .style(style::rounded_secondary_button)
                    .on_press(Message::AskDeleteHbcApp(i))
            ]
            .spacing(5)
        }),
    ];

    let table = if !state.hbc_filter.is_empty() {
        let matcher = SkimMatcherV2::default();
        let indexes = state
            .hbc_apps
            .iter()
            .enumerate()
            .filter_map(|(i, app)| {
                matcher
                    .fuzzy_match(&app.meta.name, &state.hbc_filter)
                    .map(|score| (i, score))
            })
            .sorted_unstable_by_key(|(_, score)| *score)
            .map(|(i, _)| i);

        table(t_columns, indexes).width(Length::Fill)
    } else {
        table(t_columns, 0..state.hbc_apps.len()).width(Length::Fill)
    };

    container(table).padding(10).into()
}
