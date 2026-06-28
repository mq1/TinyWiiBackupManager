// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{button, column, text},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut col = column![text("Games")].padding(10);

    for plugin in &state.plugins {
        let tool = button(plugin.contents.tools[0].name.as_str())
            .on_press_with(|| Message::RunLuaFunction(plugin.contents.tools[0].run.clone()));

        col = col.push(tool);
    }

    col.into()
}
