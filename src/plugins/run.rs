// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, plugins::plugin::Plugin, util};
use iced::task::{Straw, sipper};
use mlua::{Function, Lua, LuaSerdeExt, Table};
use sipper::Sender;

fn make_app_interface(lua: &Lua, sender: Sender<Message>) -> Result<Table, mlua::Error> {
    let twbm = lua.create_table()?;

    twbm.set(
        "notify",
        lua.create_async_function(move |lua, notification| {
            let mut sender = sender.clone();

            async move {
                let notification = lua.from_value(notification)?;
                sender.send(Message::Notify(notification)).await;
                Ok(())
            }
        })?,
    )?;

    twbm.set(
        "download_file",
        lua.create_async_function(|_lua, (uri, dest): (String, String)| async move {
            util::http::download_file(&uri, &dest).await?;
            Ok(())
        })?,
    )?;

    twbm.set("version", env!("CARGO_PKG_VERSION"))?;

    Ok(twbm)
}

pub fn run_tool(plugin: Plugin, tool_i: usize) -> impl Straw<(), Message, anyhow::Error> {
    sipper(async move |sender| {
        let lua = Lua::new();

        let twbm = make_app_interface(&lua, sender)?;
        lua.globals().set("twbm", twbm)?;

        let ctx = lua.create_table()?;

        let plugin = lua.load(&plugin.code).eval::<Table>()?;
        let tools = plugin.get::<Vec<Table>>("tools")?;
        let run = tools[tool_i].get::<Function>("run")?;

        run.call_async::<()>(ctx).await?;

        Ok(())
    })
}
