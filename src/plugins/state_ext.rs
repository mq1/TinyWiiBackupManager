// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{notifications::Notification, state::AppState, util};
use mlua::{ErrorContext, Function, Lua, LuaSerdeExt, Table};

impl AppState {
    pub fn run_tool(&mut self, plugin_i: usize, tool_i: usize) {
        let lua = Lua::new();

        let res = lua.scope(|scope| {
            let plugin = lua.load(&self.plugins[plugin_i].code).eval::<Table>()?;

            let send_message = scope.create_function_mut({
                |lua, message| {
                    let message = lua.from_value(message)?;
                    let _ = self.update(message);
                    Ok(())
                }
            })?;

            let download_file = scope.create_function_mut({
                |_lua, (uri, dest): (String, String)| {
                    util::http::download_file(&uri, &dest).map_err(Into::into)
                }
            })?;

            let twbm = lua.create_table()?;
            twbm.set("send_message", send_message)?;
            twbm.set("download_file", download_file)?;

            let tools = plugin.get::<Vec<Table>>("tools")?;
            let run = tools[tool_i].get::<Function>("run")?;
            run.call::<()>(twbm)?;

            Ok(())
        });

        if let Err(e) = res {
            let e = e.context("Failed to run lua function");
            self.notifications.add(Notification::error(e));
        }
    }
}
