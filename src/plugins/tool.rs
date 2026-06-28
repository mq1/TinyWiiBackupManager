// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use mlua::{FromLua, Function, Lua, Value};

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub group: String,
    pub run: Vec<u8>,
}

impl FromLua for Tool {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value
            .as_table()
            .ok_or(mlua::Error::runtime("expected a table"))?;

        let name = table.get("name")?;
        let description = table.get("description")?;
        let icon = table.get("icon")?;
        let group = table.get("group")?;
        let run = table.get::<Function>("run")?.dump(true);

        Ok(Tool {
            name,
            description,
            icon,
            group,
            run,
        })
    }
}
