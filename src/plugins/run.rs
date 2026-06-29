// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, plugins::plugin::Plugin, util};
use iced::task::{Straw, sipper};
use mlua::{Function, Lua, LuaSerdeExt, Table};

pub fn run_tool(plugin: Plugin, tool_i: usize) -> impl Straw<(), Message, anyhow::Error> {
    sipper(async move |sender| {
        let lua = Lua::new();

        let plugin = lua.load(&plugin.code).eval::<Table>()?;

        let send_message = lua.create_async_function(move |lua, message| {
            let mut sender = sender.clone();

            async move {
                let message = lua.from_value(message)?;
                sender.send(message).await;

                Ok(())
            }
        })?;

        let download_file = lua.create_function(|_lua, (uri, dest): (String, String)| {
            util::http::download_file(&uri, &dest)?;
            Ok(())
        })?;

        let twbm = lua.create_table()?;
        twbm.set("send_message", send_message)?;
        twbm.set("download_file", download_file)?;

        let tools = plugin.get::<Vec<Table>>("tools")?;
        let run = tools[tool_i].get::<Function>("run")?;
        run.call_async::<()>(twbm).await?;

        Ok(())
    })
}
