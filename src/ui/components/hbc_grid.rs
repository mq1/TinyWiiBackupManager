// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::components};
use iced::{Element, Length, widget::Row};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut row = Row::new().width(Length::Fill).spacing(10).padding(10);

    if !state.hbc_filter.is_empty() {
        for app in state.hbc_app_list.iter_filtered() {
            row = row.push(components::hbc_card::view(state, app));
        }
    } else {
        for app in state.hbc_app_list.iter() {
            row = row.push(components::hbc_card::view(state, app));
        }
    }

    row.wrap().into()
}
