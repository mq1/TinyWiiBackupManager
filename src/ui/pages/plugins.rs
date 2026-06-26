// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    state::AppState,
    ui::components::{plugin_card, plugins_titlebar},
};
use iced::{Element, widget::column};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut col = column![plugins_titlebar::view()].padding(10).spacing(10);

    for plugin in &state.plugins {
        col = col.push(plugin_card::view(plugin));
    }

    col.into()
}
