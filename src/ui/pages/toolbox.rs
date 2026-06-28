// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{button, column, text},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut col = column![text("Toolbox")].padding(10);

    for plugin in &state.plugins {
        for tool in &plugin.contents.tools {
            let tool = button(tool.name.as_str()).on_press(Message::RunTool(tool.id));

            col = col.push(tool);
        }
    }

    col.into()
}
