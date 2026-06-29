// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{button, column, text},
};

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let mut col = column![text("Toolbox")].padding(10);

    for (plugin_i, plugin) in state.plugins.iter().enumerate() {
        for (tool_i, tool) in plugin.meta.tools.iter().enumerate() {
            let tool = button(tool.name.as_str()).on_press(Message::RunTool(plugin_i, tool_i));

            col = col.push(tool);
        }
    }

    col.into()
}
