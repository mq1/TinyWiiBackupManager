// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, plugins::plugin::Plugin, util};
use iced::task::{Straw, sipper};
use mlua::{Function, Lua, LuaSerdeExt, Table};

fn make_app_interface<S, F>(lua: &Lua, send_message: S) -> Result<Table, mlua::Error>
where
    S: (FnMut(Message) -> F) + Clone + Send + 'static,
    F: Future<Output = ()> + Send,
{
    let twbm = lua.create_table()?;

    twbm.set(
        "notify",
        lua.create_async_function(move |lua, notification| {
            let mut send_message = send_message.clone();

            async move {
                let notification = lua.from_value(notification)?;
                send_message(Message::Notify(notification)).await;
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

        let send_message = move |message| {
            let mut sender = sender.clone();
            async move { sender.send(message).await }
        };

        let twbm = make_app_interface(&lua, send_message)?;
        lua.globals().set("twbm", twbm)?;

        let ctx = lua.create_table()?;

        let plugin = lua.load(&plugin.code).eval::<Table>()?;
        let tools = plugin.get::<Vec<Table>>("tools")?;
        let run = tools[tool_i].get::<Function>("run")?;

        run.call_async::<()>(ctx).await?;

        Ok(())
    })
}
